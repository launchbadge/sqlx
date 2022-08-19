use crate::connection::{ConnectOptions, Connection};
use crate::error::Error;
use crate::executor::Executor;
use crate::migrate::MigrateError;
use crate::migrate::{AppliedMigration, Migration};
use crate::migrate::{Migrate, MigrateDatabase};
use crate::query::query;
use crate::query_as::query_as;
use crate::query_scalar::query_scalar;
use crate::sqlite::{Sqlite, SqliteConnectOptions, SqliteConnection, SqliteJournalMode};
use futures_core::future::BoxFuture;
use sqlx_rt::fs;
use std::str::FromStr;
use std::sync::atomic::Ordering;
use std::time::Duration;
use std::time::Instant;

impl MigrateDatabase for Sqlite {
    fn create_database(url: &str) -> BoxFuture<'_, Result<(), Error>> {
        Box::pin(async move {
            let mut opts = SqliteConnectOptions::from_str(url)?.create_if_missing(true);

            // Since it doesn't make sense to include this flag in the connection URL,
            // we just use an `AtomicBool` to pass it.
            if super::CREATE_DB_WAL.load(Ordering::Acquire) {
                opts = opts.journal_mode(SqliteJournalMode::Wal);
            }

            // Opening a connection to sqlite creates the database
            let _ = opts
                .connect()
                .await?
                // Ensure WAL mode tempfiles are cleaned up
                .close()
                .await?;

            Ok(())
        })
    }

    fn database_exists(url: &str) -> BoxFuture<'_, Result<bool, Error>> {
        Box::pin(async move {
            let options = SqliteConnectOptions::from_str(url)?;

            if options.in_memory {
                Ok(true)
            } else {
                Ok(options.filename.exists())
            }
        })
    }

    fn drop_database(url: &str) -> BoxFuture<'_, Result<(), Error>> {
        Box::pin(async move {
            let options = SqliteConnectOptions::from_str(url)?;

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
                format!(
                    r#"
CREATE TABLE IF NOT EXISTS {} (
    version BIGINT PRIMARY KEY,
    description TEXT NOT NULL,
    installed_on TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    success BOOLEAN NOT NULL,
    checksum BLOB NOT NULL,
    execution_time BIGINT NOT NULL
);
                "#,
                    self.get_migrate_table_name()
                )
                .as_str(),
            )
            .await?;

            Ok(())
        })
    }

    fn version(&mut self) -> BoxFuture<'_, Result<Option<(i64, bool)>, MigrateError>> {
        Box::pin(async move {
            // language=SQLite
            let row = query_as(
                format!(
                    "SELECT version, NOT success FROM {} ORDER BY version DESC LIMIT 1",
                    self.get_migrate_table_name()
                )
                .as_str(),
            )
            .fetch_optional(self)
            .await?;

            Ok(row)
        })
    }

    fn dirty_version(&mut self) -> BoxFuture<'_, Result<Option<i64>, MigrateError>> {
        Box::pin(async move {
            // language=SQLite
            let row: Option<(i64,)> = query_as(
                format!(
                    "SELECT version FROM {} WHERE success = false ORDER BY version LIMIT 1",
                    self.get_migrate_table_name()
                )
                .as_str(),
            )
            .fetch_optional(self)
            .await?;

            Ok(row.map(|r| r.0))
        })
    }

    fn list_applied_migrations(
        &mut self,
    ) -> BoxFuture<'_, Result<Vec<AppliedMigration>, MigrateError>> {
        Box::pin(async move {
            // language=SQLite
            let rows: Vec<(i64, Vec<u8>)> = query_as(
                format!(
                    "SELECT version, checksum FROM {} ORDER BY version",
                    self.get_migrate_table_name()
                )
                .as_str(),
            )
            .fetch_all(self)
            .await?;

            let migrations = rows
                .into_iter()
                .map(|(version, checksum)| AppliedMigration {
                    version,
                    checksum: checksum.into(),
                })
                .collect();

            Ok(migrations)
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
            let checksum: Option<Vec<u8>> = query_scalar(
                format!(
                    "SELECT checksum FROM {} WHERE version = ?1",
                    self.get_migrate_table_name()
                )
                .as_str(),
            )
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
                format!(
                    r#"
    INSERT INTO {} ( version, description, success, checksum, execution_time )
    VALUES ( ?1, ?2, TRUE, ?3, ?4 )
                "#,
                    self.get_migrate_table_name()
                )
                .as_str(),
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
            let _ = query(
                format!(
                    r#"DELETE FROM {} WHERE version = ?1"#,
                    self.get_migrate_table_name()
                )
                .as_str(),
            )
            .bind(migration.version)
            .execute(self)
            .await?;

            Ok(elapsed)
        })
    }
}
