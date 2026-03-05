use std::future::Future;
use std::ops::Deref;
use std::str::FromStr;
use std::sync::OnceLock;
use std::time::Duration;

use crate::error::Error;
use crate::executor::Executor;
use crate::pool::{Pool, PoolOptions};
use crate::query::query;
use crate::{Mssql, MssqlConnectOptions, MssqlConnection};
use sqlx_core::connection::Connection;
use sqlx_core::query_scalar::query_scalar;

pub(crate) use sqlx_core::testing::*;

// Using a blocking `OnceLock` here because the critical sections are short.
static MASTER_POOL: OnceLock<Pool<Mssql>> = OnceLock::new();

impl TestSupport for Mssql {
    fn test_context(
        args: &TestArgs,
    ) -> impl Future<Output = Result<TestContext<Self>, Error>> + Send + '_ {
        test_context(args)
    }

    async fn cleanup_test(db_name: &str) -> Result<(), Error> {
        let mut conn = MASTER_POOL
            .get()
            .expect("cleanup_test() invoked outside `#[sqlx::test]`")
            .acquire()
            .await?;

        do_cleanup(&mut conn, db_name).await
    }

    async fn cleanup_test_dbs() -> Result<Option<usize>, Error> {
        let url = dotenvy::var("DATABASE_URL").expect("DATABASE_URL must be set");

        let mut conn = MssqlConnection::connect(&url).await?;

        let delete_db_names: Vec<String> =
            query_scalar("SELECT db_name FROM _sqlx_test_databases")
                .fetch_all(&mut conn)
                .await?;

        if delete_db_names.is_empty() {
            return Ok(None);
        }

        let mut deleted_count = 0usize;

        for db_name in &delete_db_names {
            match query(
                "IF DB_ID(@p1) IS NOT NULL \
                 BEGIN \
                     DECLARE @sql NVARCHAR(MAX); \
                     SET @sql = N'ALTER DATABASE ' + QUOTENAME(@p1) + N' SET SINGLE_USER WITH ROLLBACK IMMEDIATE'; \
                     EXEC sp_executesql @sql; \
                     SET @sql = N'DROP DATABASE ' + QUOTENAME(@p1); \
                     EXEC sp_executesql @sql; \
                 END",
            )
            .bind(db_name)
            .execute(&mut conn)
            .await
            {
                Ok(_deleted) => {
                    deleted_count += 1;
                }
                // Assume a database error just means the DB is still in use.
                Err(Error::Database(dbe)) => {
                    eprintln!("could not clean test database {db_name:?}: {dbe}")
                }
                // Bubble up other errors
                Err(e) => return Err(e),
            }
        }

        if deleted_count == 0 {
            return Ok(None);
        }

        // Clean up the tracking table
        for db_name in &delete_db_names {
            let _ = query("DELETE FROM _sqlx_test_databases WHERE db_name = @p1")
                .bind(db_name)
                .execute(&mut conn)
                .await;
        }

        let _ = conn.close().await;
        Ok(Some(deleted_count))
    }

    async fn snapshot(_conn: &mut Self::Connection) -> Result<FixtureSnapshot<Self>, Error> {
        Err(Error::Configuration("snapshots are not yet supported for MSSQL".into()))
    }
}

async fn test_context(args: &TestArgs) -> Result<TestContext<Mssql>, Error> {
    let url = dotenvy::var("DATABASE_URL").expect("DATABASE_URL must be set");

    let master_opts = MssqlConnectOptions::from_str(&url).expect("failed to parse DATABASE_URL");

    let pool = PoolOptions::new()
        .max_connections(20)
        // Immediately close master connections. Tokio's I/O streams don't like hopping runtimes.
        .after_release(|_conn, _| Box::pin(async move { Ok(false) }))
        .connect_lazy_with(master_opts);

    let master_pool = match once_lock_try_insert_polyfill(&MASTER_POOL, pool) {
        Ok(inserted) => inserted,
        Err((existing, pool)) => {
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

    // Create tracking table if it doesn't exist
    conn.execute(
        r#"
        IF NOT EXISTS (SELECT * FROM sys.tables WHERE name = '_sqlx_test_databases')
        CREATE TABLE _sqlx_test_databases (
            db_name NVARCHAR(200) NOT NULL PRIMARY KEY,
            test_path NVARCHAR(MAX) NOT NULL,
            created_at DATETIME2 NOT NULL DEFAULT SYSUTCDATETIME()
        );
    "#,
    )
    .await?;

    let db_name = Mssql::db_name(args);
    do_cleanup(&mut conn, &db_name).await?;

    query("INSERT INTO _sqlx_test_databases(db_name, test_path) VALUES (@p1, @p2)")
        .bind(&db_name)
        .bind(args.test_path)
        .execute(&mut *conn)
        .await?;

    query(
        "DECLARE @sql NVARCHAR(MAX) = N'CREATE DATABASE ' + QUOTENAME(@p1); \
         EXEC sp_executesql @sql;",
    )
    .bind(&db_name)
    .execute(&mut *conn)
    .await?;

    eprintln!("created database {db_name}");

    Ok(TestContext {
        pool_opts: PoolOptions::new()
            .max_connections(5)
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

async fn do_cleanup(conn: &mut MssqlConnection, db_name: &str) -> Result<(), Error> {
    query(
        "IF DB_ID(@p1) IS NOT NULL \
         BEGIN \
             DECLARE @sql NVARCHAR(MAX); \
             SET @sql = N'ALTER DATABASE ' + QUOTENAME(@p1) + N' SET SINGLE_USER WITH ROLLBACK IMMEDIATE'; \
             EXEC sp_executesql @sql; \
             SET @sql = N'DROP DATABASE ' + QUOTENAME(@p1); \
             EXEC sp_executesql @sql; \
         END",
    )
    .bind(db_name)
    .execute(&mut *conn)
    .await?;
    query("DELETE FROM _sqlx_test_databases WHERE db_name = @p1")
        .bind(db_name)
        .execute(&mut *conn)
        .await?;

    Ok(())
}

fn once_lock_try_insert_polyfill<T>(this: &OnceLock<T>, value: T) -> Result<&T, (&T, T)> {
    let mut value = Some(value);
    let res = this.get_or_init(|| value.take().unwrap());
    match value {
        None => Ok(res),
        Some(value) => Err((res, value)),
    }
}
