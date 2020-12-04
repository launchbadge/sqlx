use crate::aurora::{Aurora, AuroraConnectOptions, AuroraConnection, AuroraDbType};
use crate::connection::{ConnectOptions, Connection};
use crate::error::Error;
use crate::executor::Executor;
use crate::migrate::MigrateError;
use crate::migrate::Migration;
use crate::migrate::{Migrate, MigrateDatabase};
use crate::query::query;
use crate::query_as::query_as;
use crate::query_scalar::query_scalar;
use crc::crc32;
use futures_core::future::BoxFuture;
use std::str::FromStr;
use std::time::Duration;
use std::time::Instant;

fn parse_for_maintenance(uri: &str) -> Result<(AuroraConnectOptions, String, AuroraDbType), Error> {
    let mut options = AuroraConnectOptions::from_str(uri)?;

    let db_type = if let Some(db_type) = options.db_type {
        db_type
    } else {
        return Err(Error::Configuration(
            "DATABASE_URL does not specify a db type".into(),
        ));
    };

    let database = if let Some(database) = &options.database {
        database.to_owned()
    } else {
        return Err(Error::Configuration(
            "DATABASE_URL does not specify a database".into(),
        ));
    };

    match db_type {
        AuroraDbType::MySQL => options.database = None,
        AuroraDbType::Postgres => options.database = Some("postgres".into()),
    }

    Ok((options, database, db_type))
}

impl MigrateDatabase for Aurora {
    fn create_database(uri: &str) -> BoxFuture<'_, Result<(), Error>> {
        Box::pin(async move {
            let (options, database, db_type) = parse_for_maintenance(uri)?;
            let mut conn = options.connect().await?;

            let sql = match db_type {
                AuroraDbType::MySQL => format!("CREATE DATABASE `{}`", database),
                AuroraDbType::Postgres => {
                    format!("CREATE DATABASE \"{}\"", database.replace('"', "\"\""))
                }
            };

            let _ = conn.execute(&*sql).await?;

            Ok(())
        })
    }

    fn database_exists(uri: &str) -> BoxFuture<'_, Result<bool, Error>> {
        Box::pin(async move {
            let (options, database, db_type) = parse_for_maintenance(uri)?;
            let mut conn = options.connect().await?;

            let sql = match db_type {
                AuroraDbType::MySQL => {
                    "select exists(SELECT 1 from INFORMATION_SCHEMA.SCHEMATA WHERE SCHEMA_NAME = ?)"
                }
                AuroraDbType::Postgres => {
                    "select exists(SELECT 1 from pg_database WHERE datname = $1)"
                }
            };

            let exists: bool = query_scalar(sql)
                .bind(database)
                .fetch_one(&mut conn)
                .await?;

            Ok(exists)
        })
    }

    fn drop_database(uri: &str) -> BoxFuture<'_, Result<(), Error>> {
        Box::pin(async move {
            let (options, database, db_type) = parse_for_maintenance(uri)?;
            let mut conn = options.connect().await?;

            let sql = match db_type {
                AuroraDbType::MySQL => {
                    format!("DROP DATABASE IF EXISTS `{}`", database,)
                }
                AuroraDbType::Postgres => {
                    format!(
                        "DROP DATABASE IF EXISTS \"{}\"",
                        database.replace('"', "\"\"")
                    )
                }
            };

            let _ = conn.execute(&*sql).await?;

            Ok(())
        })
    }
}

impl Migrate for AuroraConnection {
    fn ensure_migrations_table(&mut self) -> BoxFuture<'_, Result<(), MigrateError>> {
        Box::pin(async move {
            let sql = match self.db_type {
                AuroraDbType::MySQL => {
                    r#"
                CREATE TABLE IF NOT EXISTS _sqlx_migrations (
                    version BIGINT PRIMARY KEY,
                    description TEXT NOT NULL,
                    installed_on TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
                    success BOOLEAN NOT NULL,
                    checksum BLOB NOT NULL,
                    execution_time BIGINT NOT NULL
                );
                                "#
                }
                AuroraDbType::Postgres => {
                    r#"
                CREATE TABLE IF NOT EXISTS _sqlx_migrations (
                    version BIGINT PRIMARY KEY,
                    description TEXT NOT NULL,
                    installed_on TIMESTAMPTZ NOT NULL DEFAULT now(),
                    success BOOLEAN NOT NULL,
                    checksum BYTEA NOT NULL,
                    execution_time BIGINT NOT NULL
                );
                                "#
                }
            };

            // language=SQL
            self.execute(sql).await?;

            Ok(())
        })
    }

    fn version(&mut self) -> BoxFuture<'_, Result<Option<(i64, bool)>, MigrateError>> {
        Box::pin(async move {
            // language=SQL
            let row = query_as(
                "SELECT version, NOT success FROM _sqlx_migrations ORDER BY version DESC LIMIT 1",
            )
            .fetch_optional(self)
            .await?;

            Ok(row)
        })
    }

    fn lock(&mut self) -> BoxFuture<'_, Result<(), MigrateError>> {
        Box::pin(async move {
            let database_name = current_database(self).await?;
            let lock_id = generate_lock_id(&database_name);

            let sql = match self.db_type {
                AuroraDbType::MySQL => "SELECT GET_LOCK(?, -1)",
                AuroraDbType::Postgres => "SELECT pg_advisory_lock($1)",
            };

            // language=SQL
            let query = query(sql);

            match self.db_type {
                AuroraDbType::MySQL => {
                    query.bind(format!("{:x}", lock_id)).execute(self).await?;
                }
                AuroraDbType::Postgres => {
                    query.bind(lock_id).execute(self).await?;
                }
            };

            Ok(())
        })
    }

    fn unlock(&mut self) -> BoxFuture<'_, Result<(), MigrateError>> {
        Box::pin(async move {
            let database_name = current_database(self).await?;
            let lock_id = generate_lock_id(&database_name);

            let sql = match self.db_type {
                AuroraDbType::MySQL => "SELECT RELEASE_LOCK(?)",
                AuroraDbType::Postgres => "SELECT pg_advisory_unlock($1)",
            };

            // language=SQL
            let query = query(sql);

            match self.db_type {
                AuroraDbType::MySQL => {
                    query.bind(format!("{:x}", lock_id)).execute(self).await?;
                }
                AuroraDbType::Postgres => {
                    query.bind(lock_id).execute(self).await?;
                }
            };

            Ok(())
        })
    }

    fn validate<'e: 'm, 'm>(
        &'e mut self,
        migration: &'m Migration,
    ) -> BoxFuture<'m, Result<(), MigrateError>> {
        Box::pin(async move {
            let sql = match self.db_type {
                AuroraDbType::MySQL => "SELECT checksum FROM _sqlx_migrations WHERE version = ?",
                AuroraDbType::Postgres => {
                    "SELECT checksum FROM _sqlx_migrations WHERE version = $1"
                }
            };

            // language=SQL
            let checksum: Option<Vec<u8>> = query_scalar(sql)
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
            match self.db_type {
                AuroraDbType::MySQL => {
                    let start = Instant::now();

                    let res = self.execute(&*migration.sql).await;

                    let elapsed = start.elapsed();

                    // language=MySQL
                    let _ = query(
                        r#"
            INSERT INTO _sqlx_migrations ( version, description, success, checksum, execution_time )
            VALUES ( ?, ?, ?, ?, ? )
                        "#,
                    )
                    .bind(migration.version)
                    .bind(&*migration.description)
                    .bind(res.is_ok())
                    .bind(&*migration.checksum)
                    .bind(elapsed.as_nanos() as i64)
                    .execute(self)
                    .await?;

                    res?;

                    Ok(elapsed)
                }
                AuroraDbType::Postgres => {
                    let mut tx = self.begin().await?;
                    let start = Instant::now();

                    let _ = tx.execute(&*migration.sql).await?;

                    tx.commit().await?;

                    let elapsed = start.elapsed();

                    // language=SQL
                    let _ = query(
                        r#"
            INSERT INTO _sqlx_migrations ( version, description, success, checksum, execution_time )
            VALUES ( $1, $2, TRUE, $3, $4 )
                        "#,
                    )
                    .bind(migration.version)
                    .bind(&*migration.description)
                    .bind(&*migration.checksum)
                    .bind(elapsed.as_nanos() as i64)
                    .execute(self)
                    .await?;

                    Ok(elapsed)
                }
            }
        })
    }

    fn revert<'e: 'm, 'm>(
        &'e mut self,
        migration: &'m Migration,
    ) -> BoxFuture<'m, Result<Duration, MigrateError>> {
        Box::pin(async move {
            match self.db_type {
                AuroraDbType::MySQL => {
                    let start = Instant::now();

                    self.execute(&*migration.sql).await?;

                    let elapsed = start.elapsed();

                    // language=SQL
                    let _ = query(r#"DELETE FROM _sqlx_migrations WHERE version = ?"#)
                        .bind(migration.version)
                        .execute(self)
                        .await?;

                    Ok(elapsed)
                }
                AuroraDbType::Postgres => {
                    let mut tx = self.begin().await?;
                    let start = Instant::now();

                    let _ = tx.execute(&*migration.sql).await?;

                    tx.commit().await?;

                    let elapsed = start.elapsed();

                    // language=SQL
                    let _ = query(r#"DELETE FROM _sqlx_migrations WHERE version = $1"#)
                        .bind(migration.version)
                        .execute(self)
                        .await?;

                    Ok(elapsed)
                }
            }
        })
    }
}

async fn current_database(conn: &mut AuroraConnection) -> Result<String, MigrateError> {
    let sql = match conn.db_type {
        AuroraDbType::MySQL => "SELECT DATABASE()",
        AuroraDbType::Postgres => "SELECT current_database()",
    };

    // language=SQL
    Ok(query_scalar(sql).fetch_one(conn).await?)
}

// inspired from rails: https://github.com/rails/rails/blob/6e49cc77ab3d16c06e12f93158eaf3e507d4120e/activerecord/lib/active_record/migration.rb#L1308
fn generate_lock_id(database_name: &str) -> i64 {
    // 0x3d32ad9e chosen by fair dice roll
    0x3d32ad9e * (crc32::checksum_ieee(database_name.as_bytes()) as i64)
}
