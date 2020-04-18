use anyhow::Result;
use async_trait::async_trait;

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
