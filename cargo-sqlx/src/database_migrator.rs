use async_trait::async_trait;
use anyhow::{Result};

#[async_trait]
pub trait DatabaseMigrator {
    fn database_type(&self) -> String;

    fn get_database_name(&self) -> Result<String>;

    fn can_migrate_database(&self) -> bool;
    fn can_create_database(&self) -> bool;
    fn can_drop_database(&self) -> bool;

    async fn check_if_database_exists(&self, db_name: &str) -> Result<bool>;
    async fn create_database(&self, db_name: &str) -> Result<()>;
    async fn drop_database(&self, db_name: &str) -> Result<()>;
}
