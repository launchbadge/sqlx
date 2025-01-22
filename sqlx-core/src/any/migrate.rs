use crate::any::driver;
use crate::any::{Any, AnyConnection};
use crate::error::Error;
use crate::migrate::{AppliedMigration, Migrate, MigrateDatabase, MigrateError, Migration};
use futures_core::future::BoxFuture;
use std::time::Duration;

impl MigrateDatabase for Any {
    fn create_database(url: &str) -> BoxFuture<'_, Result<(), Error>> {
        Box::pin(async {
            driver::from_url_str(url)?
                .get_migrate_database()?
                .create_database(url)
                .await
        })
    }

    fn database_exists(url: &str) -> BoxFuture<'_, Result<bool, Error>> {
        Box::pin(async {
            driver::from_url_str(url)?
                .get_migrate_database()?
                .database_exists(url)
                .await
        })
    }

    fn drop_database(url: &str) -> BoxFuture<'_, Result<(), Error>> {
        Box::pin(async {
            driver::from_url_str(url)?
                .get_migrate_database()?
                .drop_database(url)
                .await
        })
    }

    fn force_drop_database(url: &str) -> BoxFuture<'_, Result<(), Error>> {
        Box::pin(async {
            driver::from_url_str(url)?
                .get_migrate_database()?
                .force_drop_database(url)
                .await
        })
    }
}

impl Migrate for AnyConnection {
    fn create_schema_if_not_exists<'e>(
        &'e mut self,
        schema_name: &'e str,
    ) -> BoxFuture<'e, Result<(), MigrateError>> {
        Box::pin(async {
            self.get_migrate()?
                .create_schema_if_not_exists(schema_name)
                .await
        })
    }

    fn ensure_migrations_table<'e>(
        &'e mut self,
        table_name: &'e str,
    ) -> BoxFuture<'e, Result<(), MigrateError>> {
        Box::pin(async {
            self.get_migrate()?
                .ensure_migrations_table(table_name)
                .await
        })
    }

    fn dirty_version<'e>(
        &'e mut self,
        table_name: &'e str,
    ) -> BoxFuture<'e, Result<Option<i64>, MigrateError>> {
        Box::pin(async { self.get_migrate()?.dirty_version(table_name).await })
    }

    fn list_applied_migrations<'e>(
        &'e mut self,
        table_name: &'e str,
    ) -> BoxFuture<'e, Result<Vec<AppliedMigration>, MigrateError>> {
        Box::pin(async {
            self.get_migrate()?
                .list_applied_migrations(table_name)
                .await
        })
    }

    fn lock(&mut self) -> BoxFuture<'_, Result<(), MigrateError>> {
        Box::pin(async { self.get_migrate()?.lock().await })
    }

    fn unlock(&mut self) -> BoxFuture<'_, Result<(), MigrateError>> {
        Box::pin(async { self.get_migrate()?.unlock().await })
    }

    fn apply<'e>(
        &'e mut self,
        table_name: &'e str,
        migration: &'e Migration,
    ) -> BoxFuture<'e, Result<Duration, MigrateError>> {
        Box::pin(async { self.get_migrate()?.apply(table_name, migration).await })
    }

    fn revert<'e>(
        &'e mut self,
        table_name: &'e str,
        migration: &'e Migration,
    ) -> BoxFuture<'e, Result<Duration, MigrateError>> {
        Box::pin(async { self.get_migrate()?.revert(table_name, migration).await })
    }
}
