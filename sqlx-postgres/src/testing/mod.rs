use std::fmt::Write;
use std::str::FromStr;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::{Duration, SystemTime};

use futures_core::future::BoxFuture;

use once_cell::sync::OnceCell;

use crate::connection::Connection;

use crate::error::Error;
use crate::executor::Executor;
use crate::pool::{Pool, PoolOptions};
use crate::query::query;
use crate::query_scalar::query_scalar;
use crate::{PgConnectOptions, PgConnection, Postgres};

pub(crate) use sqlx_core::testing::*;

// Using a blocking `OnceCell` here because the critical sections are short.
static MASTER_POOL: OnceCell<Pool<Postgres>> = OnceCell::new();
// Automatically delete any databases created before the start of the test binary.
static DO_CLEANUP: AtomicBool = AtomicBool::new(true);

impl TestSupport for Postgres {
    fn test_context(args: &TestArgs) -> BoxFuture<'_, Result<TestContext<Self>, Error>> {
        Box::pin(async move { test_context(args).await })
    }

    fn cleanup_test(db_name: &str) -> BoxFuture<'_, Result<(), Error>> {
        Box::pin(async move {
            let mut conn = MASTER_POOL
                .get()
                .expect("cleanup_test() invoked outside `#[sqlx::test]")
                .acquire()
                .await?;

            conn.execute(&format!("drop database if exists {db_name:?};")[..])
                .await?;

            query("delete from _sqlx_test.databases where db_name = $1")
                .bind(db_name)
                .execute(&mut *conn)
                .await?;

            Ok(())
        })
    }

    fn cleanup_test_dbs() -> BoxFuture<'static, Result<Option<usize>, Error>> {
        Box::pin(async move {
            let url = dotenvy::var("DATABASE_URL").expect("DATABASE_URL must be set");

            let mut conn = PgConnection::connect(&url).await?;

            let now = SystemTime::now()
                .duration_since(SystemTime::UNIX_EPOCH)
                .unwrap();

            let num_deleted = do_cleanup(&mut conn, now).await?;
            let _ = conn.close().await;
            Ok(Some(num_deleted))
        })
    }

    fn snapshot(
        _conn: &mut Self::Connection,
    ) -> BoxFuture<'_, Result<FixtureSnapshot<Self>, Error>> {
        // TODO: I want to get the testing feature out the door so this will have to wait,
        // but I'm keeping the code around for now because I plan to come back to it.
        todo!()
    }
}

async fn test_context(args: &TestArgs) -> Result<TestContext<Postgres>, Error> {
    let url = dotenvy::var("DATABASE_URL").expect("DATABASE_URL must be set");

    let master_opts = PgConnectOptions::from_str(&url).expect("failed to parse DATABASE_URL");

    let pool = PoolOptions::new()
        // Postgres' normal connection limit is 100 plus 3 superuser connections
        // We don't want to use the whole cap and there may be fuzziness here due to
        // concurrently running tests anyway.
        .max_connections(20)
        // Immediately close master connections. Tokio's I/O streams don't like hopping runtimes.
        .after_release(|_conn, _| Box::pin(async move { Ok(false) }))
        .connect_lazy_with(master_opts.clone());

    let master_pool = MASTER_POOL
        .try_insert(pool)
        .unwrap_or_else(|(existing, _pool)| existing);

    let mut conn = master_pool.acquire().await?;

    // language=PostgreSQL
    conn.execute(
        // Explicit lock avoids this latent bug: https://stackoverflow.com/a/29908840
        // I couldn't find a bug on the mailing list for `CREATE SCHEMA` specifically,
        // but a clearly related bug with `CREATE TABLE` has been known since 2007:
        // https://www.postgresql.org/message-id/200710222037.l9MKbCJZ098744%40wwwmaster.postgresql.org
        r#"
        lock table pg_catalog.pg_namespace in share row exclusive mode;

        create schema if not exists _sqlx_test;

        create table if not exists _sqlx_test.databases (
            db_name text primary key,
            test_path text not null,
            created_at timestamptz not null default now()
        );

        create index if not exists databases_created_at 
            on _sqlx_test.databases(created_at);

        create sequence if not exists _sqlx_test.database_ids;
    "#,
    )
    .await?;

    // Record the current time _before_ we acquire the `DO_CLEANUP` permit. This
    // prevents the first test thread from accidentally deleting new test dbs
    // created by other test threads if we're a bit slow.
    let now = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap();

    // Only run cleanup if the test binary just started.
    if DO_CLEANUP.swap(false, Ordering::SeqCst) {
        do_cleanup(&mut conn, now).await?;
    }

    let new_db_name: String = query_scalar(
        r#"
            insert into _sqlx_test.databases(db_name, test_path)
            select '_sqlx_test_' || nextval('_sqlx_test.database_ids'), $1
            returning db_name
        "#,
    )
    .bind(args.test_path)
    .fetch_one(&mut *conn)
    .await?;

    conn.execute(&format!("create database {new_db_name:?}")[..])
        .await?;

    Ok(TestContext {
        pool_opts: PoolOptions::new()
            // Don't allow a single test to take all the connections.
            // Most tests shouldn't require more than 5 connections concurrently,
            // or else they're likely doing too much in one test.
            .max_connections(5)
            // Close connections ASAP if left in the idle queue.
            .idle_timeout(Some(Duration::from_secs(1)))
            .parent(master_pool.clone()),
        connect_opts: master_opts.database(&new_db_name),
        db_name: new_db_name,
    })
}

async fn do_cleanup(conn: &mut PgConnection, created_before: Duration) -> Result<usize, Error> {
    // since SystemTime is not monotonic we added a little margin here to avoid race conditions with other threads
    let created_before = i64::try_from(created_before.as_secs()).unwrap() - 2;

    let delete_db_names: Vec<String> = query_scalar(
        "select db_name from _sqlx_test.databases \
            where created_at < (to_timestamp($1) at time zone 'UTC')",
    )
    .bind(created_before)
    .fetch_all(&mut *conn)
    .await?;

    if delete_db_names.is_empty() {
        return Ok(0);
    }

    let mut deleted_db_names = Vec::with_capacity(delete_db_names.len());
    let delete_db_names = delete_db_names.into_iter();

    let mut command = String::new();

    for db_name in delete_db_names {
        command.clear();
        writeln!(command, "drop database if exists {db_name:?};").ok();
        match conn.execute(&*command).await {
            Ok(_deleted) => {
                deleted_db_names.push(db_name);
            }
            // Assume a database error just means the DB is still in use.
            Err(Error::Database(dbe)) => {
                eprintln!("could not clean test database {db_name:?}: {dbe}")
            }
            // Bubble up other errors
            Err(e) => return Err(e),
        }
    }

    query("delete from _sqlx_test.databases where db_name = any($1::text[])")
        .bind(&deleted_db_names)
        .execute(&mut *conn)
        .await?;

    Ok(deleted_db_names.len())
}
