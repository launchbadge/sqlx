use sqlx::postgres::PgRow;
use sqlx::Connect;
use sqlx::PgConnection;
use sqlx::Row;

use async_trait::async_trait;
use anyhow::{anyhow, Context, Result};

use crate::database_migrator::DatabaseMigrator;

pub struct Sqlite<'a> {
    pub db_url: &'a str,
}

struct DbUrl<'a> {
    base_url: &'a str,
    db_name: &'a str,
}

fn get_base_url<'a>(db_url: &'a str) -> Result<DbUrl> {
    let split: Vec<&str> = db_url.rsplitn(2, '/').collect();

    if split.len() != 2 {
        return Err(anyhow!("Failed to find database name in connection string"));
    }

    let db_name = split[0];
    let base_url = split[1];

    Ok(DbUrl { base_url, db_name })
}


#[async_trait]
impl DatabaseMigrator for Sqlite<'_> {
    fn database_type(&self) -> String {
        "Sqlite".to_string()
    }

    fn can_migrate_database(&self) -> bool {
        false
    }

    fn can_create_database(&self) -> bool {
        false
    }

    fn can_drop_database(&self) -> bool {
        false
    }

    fn get_database_name(&self) -> Result<String> {
        let db_url = get_base_url(self.db_url)?;
        Ok(db_url.db_name.to_string())
    }

    async fn check_if_database_exists(&self, db_name: &str) -> Result<bool> {
        let db_url = get_base_url(self.db_url)?;

        let base_url = db_url.base_url;

        let mut conn = PgConnection::connect(base_url).await?;

        let result: bool =
            sqlx::query("select exists(SELECT 1 from pg_database WHERE datname = $1) as exists")
                .bind(db_name)
                .try_map(|row: PgRow| row.try_get("exists"))
                .fetch_one(&mut conn)
                .await
                .context("Failed to check if database exists")?;

        Ok(result)
    }

    async fn create_database(&self, db_name: &str) -> Result<()> {
        let db_url = get_base_url(self.db_url)?;

        let base_url = db_url.base_url;

        let mut conn = PgConnection::connect(base_url).await?;

        sqlx::query(&format!("CREATE DATABASE {}", db_name))
            .execute(&mut conn)
            .await
            .with_context(|| format!("Failed to create database: {}", db_name))?;

        Ok(())
    }

    async fn drop_database(&self, db_name: &str) -> Result<()> {
        let db_url = get_base_url(self.db_url)?;

        let base_url = db_url.base_url;

        let mut conn = PgConnection::connect(base_url).await?;

        sqlx::query(&format!("DROP DATABASE {}", db_name))
            .execute(&mut conn)
            .await
            .with_context(|| format!("Failed to create database: {}", db_name))?;

        Ok(())
    }
}
