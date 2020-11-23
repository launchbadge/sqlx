use crate::connection::{ConnectOptions, Connection};
use crate::error::Error;
use crate::executor::Executor;
use crate::migrate::MigrateError;
use crate::migrate::Migration;
use crate::migrate::{Migrate, MigrateDatabase};
use crate::query::query;
use crate::query_as::query_as;
use crate::query_scalar::query_scalar;
use crate::sqlite::{Sqlite, SqliteConnectOptions, SqliteConnection};
use futures_core::future::BoxFuture;
use sqlx_rt::fs;
use std::str::FromStr;
use std::time::Duration;
use std::time::Instant;

impl MigrateDatabase for Sqlite {
    fn create_database(uri: &str) -> BoxFuture<'_, Result<(), Error>> {
        Box::pin(async move {
            // Opening a connection to sqlite creates the database
            let _ = SqliteConnectOptions::from_str(uri)?
                .create_if_missing(true)
                .connect()
                .await?;

            Ok(())
        })
    }

    fn database_exists(uri: &str) -> BoxFuture<'_, Result<bool, Error>> {
        Box::pin(async move {
            let options = SqliteConnectOptions::from_str(uri)?;

            if options.in_memory {
                Ok(true)
            } else {
                Ok(options.filename.exists())
            }
        })
    }

    fn drop_database(uri: &str) -> BoxFuture<'_, Result<(), Error>> {
        Box::pin(async move {
            let options = SqliteConnectOptions::from_str(uri)?;

            if !options.in_memory {
                fs::remove_file(&*options.filename).await?;
            }

            Ok(())
        })
    }
}

impl Migrate for SqliteConnection {
    fn ensure_migrations_table(&mut self) -> BoxFuture<'_, Result<(), MigrateError>> {
        Box::pin(async move {
            // language=SQLite
            self.execute(
                r#"
CREATE TABLE IF NOT EXISTS _sqlx_migrations (
    version BIGINT PRIMARY KEY,
    description TEXT NOT NULL,
    installed_on TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    success BOOLEAN NOT NULL,
    checksum BLOB NOT NULL,
    execution_time BIGINT NOT NULL
);
                "#,
            )
            .await?;

            Ok(())
        })
    }

    fn version(&mut self) -> BoxFuture<'_, Result<Option<(i64, bool)>, MigrateError>> {
        Box::pin(async move {
            // language=SQLite
            let row = query_as(
                "SELECT version, NOT success FROM _sqlx_migrations ORDER BY version DESC LIMIT 1",
            )
            .fetch_optional(self)
            .await?;

            Ok(row)
        })
    }

    fn lock(&mut self) -> BoxFuture<'_, Result<(), MigrateError>> {
        Box::pin(async move { Ok(()) })
    }

    fn unlock(&mut self) -> BoxFuture<'_, Result<(), MigrateError>> {
        Box::pin(async move { Ok(()) })
    }

    fn validate<'e: 'm, 'm>(
        &'e mut self,
        migration: &'m Migration,
    ) -> BoxFuture<'m, Result<(), MigrateError>> {
        Box::pin(async move {
            // language=SQL
            let checksum: Option<Vec<u8>> =
                query_scalar("SELECT checksum FROM _sqlx_migrations WHERE version = ?1")
                    .bind(migration.version)
                    .fetch_optional(self)
                    .await?;

            if let Some(checksum) = checksum {
                if checksum == &*migration.checksum {
                    Ok(())
                } else {
                    Err(MigrateError::VersionMismatch(migration.version))
                }
            } else {
                Err(MigrateError::VersionMissing(migration.version))
            }
        })
    }

    fn apply<'e: 'm, 'm>(
        &'e mut self,
        migration: &'m Migration,
    ) -> BoxFuture<'m, Result<Duration, MigrateError>> {
        Box::pin(async move {
            let mut tx = self.begin().await?;
            let start = Instant::now();

            let _ = tx.execute(&*migration.sql).await?;

            tx.commit().await?;

            let elapsed = start.elapsed();

            // language=SQL
            let _ = query(
                r#"
    INSERT INTO _sqlx_migrations ( version, description, success, checksum, execution_time )
    VALUES ( ?1, ?2, TRUE, ?3, ?4 )
                "#,
            )
            .bind(migration.version)
            .bind(&*migration.description)
            .bind(&*migration.checksum)
            .bind(elapsed.as_nanos() as i64)
            .execute(self)
            .await?;

            Ok(elapsed)
        })
    }

    fn revert<'e: 'm, 'm>(
        &'e mut self,
        migration: &'m Migration,
    ) -> BoxFuture<'m, Result<Duration, MigrateError>> {
        Box::pin(async move {
            let mut tx = self.begin().await?;
            let start = Instant::now();

            let _ = tx.execute(&*migration.sql).await?;

            tx.commit().await?;

            let elapsed = start.elapsed();

            // language=SQL
            let _ = query(r#"DELETE FROM _sqlx_migrations WHERE version = ?1"#)
                .bind(migration.version)
                .execute(self)
                .await?;

            Ok(elapsed)
        })
    }
}
