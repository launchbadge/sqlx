use anyhow::{bail, Context, Result};
use async_trait::async_trait;
use std::env;
use url::Url;

#[cfg(feature = "postgres")]
mod postgres;

#[cfg(feature = "sqlite")]
mod sqlite;

#[async_trait]
pub trait MigrationTransaction {
    async fn commit(self: Box<Self>) -> Result<()>;
    async fn rollback(self: Box<Self>) -> Result<()>;
    async fn check_if_applied(&mut self, migration: &str) -> Result<bool>;
    async fn execute_migration(&mut self, migration_sql: &str) -> Result<()>;
    async fn save_applied_migration(&mut self, migration_name: &str) -> Result<()>;
}

#[async_trait]
pub trait DatabaseMigrator {
    // Misc info
    fn database_type(&self) -> String;
    fn get_database_name(&self) -> Result<String>;

    // Features
    fn can_migrate_database(&self) -> bool;
    fn can_create_database(&self) -> bool;
    fn can_drop_database(&self) -> bool;

    // Database creation
    async fn check_if_database_exists(&self, db_name: &str) -> Result<bool>;
    async fn create_database(&self, db_name: &str) -> Result<()>;
    async fn drop_database(&self, db_name: &str) -> Result<()>;

    // Migration
    async fn create_migration_table(&self) -> Result<()>;
    async fn get_migrations(&self) -> Result<Vec<String>>;
    async fn begin_migration(&self) -> Result<Box<dyn MigrationTransaction>>;
}

pub fn get() -> Result<Box<dyn DatabaseMigrator>> {
    let db_url_raw = env::var("DATABASE_URL").context("Failed to find 'DATABASE_URL'")?;

    let db_url = Url::parse(&db_url_raw)?;

    // This code is taken from: https://github.com/launchbadge/sqlx/blob/master/sqlx-macros/src/lib.rs#L63
    match db_url.scheme() {
        #[cfg(feature = "sqlite")]
        "sqlite" => Ok(Box::new(self::sqlite::Sqlite::new(db_url_raw ))),
        #[cfg(not(feature = "sqlite"))]
        "sqlite" => bail!("Not implemented. DATABASE_URL {} has the scheme of a SQLite database but the `sqlite` feature of sqlx was not enabled",
                            db_url),

        #[cfg(feature = "postgres")]
        "postgresql" | "postgres" => Ok(Box::new(self::postgres::Postgres::new(db_url_raw))),
        #[cfg(not(feature = "postgres"))]
        "postgresql" | "postgres" => bail!("DATABASE_URL {} has the scheme of a Postgres database but the `postgres` feature of sqlx was not enabled",
                db_url),

        #[cfg(feature = "mysql")]
        "mysql" | "mariadb" => bail!("Not implemented"),
        #[cfg(not(feature = "mysql"))]
        "mysql" | "mariadb" => bail!(
            "DATABASE_URL {} has the scheme of a MySQL/MariaDB database but the `mysql` feature of sqlx was not enabled",
             db_url
        ),

        scheme => bail!("unexpected scheme {:?} in DATABASE_URL {}", scheme, db_url),
    }
}
