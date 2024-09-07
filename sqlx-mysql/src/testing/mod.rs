use std::ops::Deref;
use std::str::FromStr;
use std::time::Duration;

use futures_core::future::BoxFuture;

use once_cell::sync::OnceCell;

use crate::error::Error;
use crate::executor::Executor;
use crate::pool::{Pool, PoolOptions};
use crate::query::query;
use crate::{MySql, MySqlConnectOptions, MySqlConnection};

pub(crate) use sqlx_core::testing::*;

// Using a blocking `OnceCell` here because the critical sections are short.
static MASTER_POOL: OnceCell<Pool<MySql>> = OnceCell::new();

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

            do_cleanup(&mut conn, db_name).await
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
        .connect_lazy_with(master_opts);

    let master_pool = match MASTER_POOL.try_insert(pool) {
        Ok(inserted) => inserted,
        Err((existing, pool)) => {
            // Sanity checks.
            assert_eq!(
                existing.connect_options().host,
                pool.connect_options().host,
                "DATABASE_URL changed at runtime, host differs"
            );

            assert_eq!(
                existing.connect_options().database,
                pool.connect_options().database,
                "DATABASE_URL changed at runtime, database differs"
            );

            existing
        }
    };

    let mut conn = master_pool.acquire().await?;

    // language=MySQL
    conn.execute(
        r#"
        create table if not exists _sqlx_test_databases (
            db_name text primary key,
            test_path text not null,
            created_at timestamp not null default current_timestamp
        );
    "#,
    )
    .await?;

    let db_name = MySql::db_name(args);
    do_cleanup(&mut conn, &db_name).await?;

    query("insert into _sqlx_test_databases(test_path) values ($1, $2)")
        .bind(&db_name)
        .bind(args.test_path)
        .execute(&mut *conn)
        .await?;

    conn.execute(&format!("create database {db_name}")[..])
        .await?;

    eprintln!("created database {db_name}");

    Ok(TestContext {
        pool_opts: PoolOptions::new()
            // Don't allow a single test to take all the connections.
            // Most tests shouldn't require more than 5 connections concurrently,
            // or else they're likely doing too much in one test.
            .max_connections(5)
            // Close connections ASAP if left in the idle queue.
            .idle_timeout(Some(Duration::from_secs(1)))
            .parent(master_pool.clone()),
        connect_opts: master_pool
            .connect_options()
            .deref()
            .clone()
            .database(&db_name),
        db_name,
    })
}

async fn do_cleanup(conn: &mut MySqlConnection, db_name: &str) -> Result<(), Error> {
    let delete_db_command = format!("drop database if exists {db_name:?};");
    conn.execute(&*delete_db_command).await?;
    query("delete from _sqlx_test.databases where db_name = $1::text")
        .bind(&db_name)
        .execute(&mut *conn)
        .await?;

    Ok(())
}
