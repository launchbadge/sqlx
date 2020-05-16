// use sqlx::pool::PoolConnection;
use sqlx::sqlite::SqliteRow;
use sqlx::Connect;
use sqlx::Executor;
use sqlx::Row;
use sqlx::SqliteConnection;
// use sqlx::SqlitePool;

use anyhow::{anyhow, Context, Result};
use async_trait::async_trait;

use crate::migrator::{DatabaseMigrator, MigrationTransaction};

pub struct Sqlite {
    db_url: String,
    path: String,
}

impl Sqlite {
    pub fn new(db_url: String) -> Self {
        let path = crop_letters(&db_url, "sqlite://".len());
        Sqlite {
            db_url: db_url.clone(),
            path: path.to_string(),
        }
    }
}

fn crop_letters(s: &str, pos: usize) -> &str {
    match s.char_indices().skip(pos).next() {
        Some((pos, _)) => &s[pos..],
        None => "",
    }
}

#[async_trait]
impl DatabaseMigrator for Sqlite {
    fn database_type(&self) -> String {
        "Sqlite".to_string()
    }

    fn can_migrate_database(&self) -> bool {
        true
    }

    fn can_create_database(&self) -> bool {
        true
    }

    fn can_drop_database(&self) -> bool {
        true
    }

    fn get_database_name(&self) -> Result<String> {
        let split: Vec<&str> = self.db_url.rsplitn(2, '/').collect();

        if split.len() != 2 {
            return Err(anyhow!("Failed to find database name in connection string"));
        }

        let db_name = split[0];

        Ok(db_name.to_string())
    }

    async fn check_if_database_exists(&self, _db_name: &str) -> Result<bool> {
        use std::path::Path;
        Ok(Path::new(&self.path).exists())
    }

    async fn create_database(&self, _db_name: &str) -> Result<()> {
        println!("DB {}", self.path);

        // Opening a connection to sqlite creates the database.
        let _ = SqliteConnection::connect(&self.db_url).await?;

        Ok(())
    }

    async fn drop_database(&self, _db_name: &str) -> Result<()> {
        std::fs::remove_file(&self.path)?;
        Ok(())
    }

    async fn create_migration_table(&self) -> Result<()> {
        let mut conn = SqliteConnection::connect(&self.db_url).await?;

        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS __migrations (
                migration TEXT PRIMARY KEY,
                created TIMESTAMP NOT NULL DEFAULT current_timestamp
            );
        "#,
        )
        .execute(&mut conn)
        .await
        .context("Failed to create migration table")?;

        Ok(())
    }

    async fn get_migrations(&self) -> Result<Vec<String>> {
        let mut conn = SqliteConnection::connect(&self.db_url).await?;

        let result = sqlx::query(
            r#"
            select migration from __migrations order by created
        "#,
        )
        .try_map(|row: SqliteRow| row.try_get(0))
        .fetch_all(&mut conn)
        .await
        .context("Failed to create migration table")?;

        Ok(result)
    }

    async fn begin_migration(&self) -> Result<Box<dyn MigrationTransaction>> {
        // let pool = SqlitePool::new(&self.db_url)
        //     .await
        //     .context("Failed to connect to pool")?;

        // let tx = pool.begin().await?;

        // Ok(Box::new(MigrationTransaction { transaction: tx }))
        Ok(Box::new(SqliteMigration {
            db_url: self.db_url.clone(),
        }))
    }
}

pub struct SqliteMigration {
    db_url: String,
    // pub transaction: sqlx::Transaction<PoolConnection<SqliteConnection>>,
}

#[async_trait]
impl MigrationTransaction for SqliteMigration {
    async fn commit(self: Box<Self>) -> Result<()> {
        // self.transaction.commit().await?;
        Ok(())
    }

    async fn rollback(self: Box<Self>) -> Result<()> {
        // self.transaction.rollback().await?;
        Ok(())
    }

    async fn check_if_applied(&mut self, migration_name: &str) -> Result<bool> {
        let mut conn = SqliteConnection::connect(&self.db_url).await?;

        let result =
            sqlx::query("select exists(select migration from __migrations where migration = $1)")
                .bind(migration_name.to_string())
                .try_map(|row: SqliteRow| row.try_get(0))
                .fetch_one(&mut conn)
                .await?;

        Ok(result)
    }

    async fn execute_migration(&mut self, migration_sql: &str) -> Result<()> {
        let mut conn = SqliteConnection::connect(&self.db_url).await?;
        conn.execute(migration_sql).await?;
        // self.transaction.execute(migration_sql).await?;
        Ok(())
    }

    async fn save_applied_migration(&mut self, migration_name: &str) -> Result<()> {
        let mut conn = SqliteConnection::connect(&self.db_url).await?;
        sqlx::query("insert into __migrations (migration) values ($1)")
            .bind(migration_name.to_string())
            .execute(&mut conn)
            .await?;
        Ok(())
    }
}
