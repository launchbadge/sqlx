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
use crate::query_builder::QueryBuilder;
use crate::query_scalar::query_scalar;
use crate::{MySql, MySqlConnectOptions, MySqlConnection};

pub(crate) use sqlx_core::testing::*;

// Using a blocking `OnceCell` here because the critical sections are short.
static MASTER_POOL: OnceCell<Pool<MySql>> = OnceCell::new();
// Automatically delete any databases created before the start of the test binary.
static DO_CLEANUP: AtomicBool = AtomicBool::new(true);

impl TestSupport for MySql {
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

            let db_id = db_id(db_name);

            conn.execute(&format!("drop database if exists {db_name};")[..])
                .await?;

            query("delete from _sqlx_test_databases where db_id = ?")
                .bind(db_id)
                .execute(&mut *conn)
                .await?;

            Ok(())
        })
    }

    fn cleanup_test_dbs() -> BoxFuture<'static, Result<Option<usize>, Error>> {
        Box::pin(async move {
            let url = dotenvy::var("DATABASE_URL").expect("DATABASE_URL must be set");

            let mut conn = MySqlConnection::connect(&url).await?;

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

async fn test_context(args: &TestArgs) -> Result<TestContext<MySql>, Error> {
    let url = dotenvy::var("DATABASE_URL").expect("DATABASE_URL must be set");

    let master_opts = MySqlConnectOptions::from_str(&url).expect("failed to parse DATABASE_URL");

    let pool = PoolOptions::new()
        // MySql's normal connection limit is 150 plus 1 superuser connection
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

    // language=MySQL
    conn.execute(
        r#"
        create table if not exists _sqlx_test_databases (
            db_id bigint unsigned primary key auto_increment,
            test_path text not null,
            created_at timestamp not null default current_timestamp
        );
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

    query("insert into _sqlx_test_databases(test_path) values (?)")
        .bind(args.test_path)
        .execute(&mut *conn)
        .await?;

    // MySQL doesn't have `INSERT ... RETURNING`
    let new_db_id: u64 = query_scalar("select last_insert_id()")
        .fetch_one(&mut *conn)
        .await?;

    let new_db_name = db_name(new_db_id);

    conn.execute(&format!("create database {new_db_name}")[..])
        .await?;

    eprintln!("created database {new_db_name}");

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

async fn do_cleanup(conn: &mut MySqlConnection, created_before: Duration) -> Result<usize, Error> {
    // since SystemTime is not monotonic we added a little margin here to avoid race conditions with other threads
    let created_before_as_secs = created_before.as_secs() - 2;
    let delete_db_ids: Vec<u64> = query_scalar(
        "select db_id from _sqlx_test_databases \
            where created_at < from_unixtime(?)",
    )
    .bind(created_before_as_secs)
    .fetch_all(&mut *conn)
    .await?;

    if delete_db_ids.is_empty() {
        return Ok(0);
    }

    let mut deleted_db_ids = Vec::with_capacity(delete_db_ids.len());

    let mut command = String::new();

    for db_id in delete_db_ids {
        command.clear();

        let db_name = db_name(db_id);

        writeln!(command, "drop database if exists {db_name}").ok();
        match conn.execute(&*command).await {
            Ok(_deleted) => {
                deleted_db_ids.push(db_id);
            }
            // Assume a database error just means the DB is still in use.
            Err(Error::Database(dbe)) => {
                eprintln!("could not clean test database {db_id:?}: {dbe}")
            }
            // Bubble up other errors
            Err(e) => return Err(e),
        }
    }

    let mut query = QueryBuilder::new("delete from _sqlx_test_databases where db_id in (");

    let mut separated = query.separated(",");

    for db_id in &deleted_db_ids {
        separated.push_bind(db_id);
    }

    query.push(")").build().execute(&mut *conn).await?;

    Ok(deleted_db_ids.len())
}

fn db_name(id: u64) -> String {
    format!("_sqlx_test_database_{id}")
}

fn db_id(name: &str) -> u64 {
    name.trim_start_matches("_sqlx_test_database_")
        .parse()
        .unwrap_or_else(|_1| panic!("failed to parse ID from database name {name:?}"))
}

#[test]
fn test_db_name_id() {
    assert_eq!(db_name(12345), "_sqlx_test_database_12345");
    assert_eq!(db_id("_sqlx_test_database_12345"), 12345);
}
