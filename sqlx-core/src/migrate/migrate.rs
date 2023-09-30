use crate::error::Result;
use crate::migrate::{AppliedMigration, MigrateResult, Migration};
use futures_core::future::BoxFuture;
use std::time::Duration;

pub trait MigrateDatabase {
    // create database in url
    // uses a maintenance database depending on driver
    fn create_database(url: &str) -> BoxFuture<'_, Result<()>>;

    // check if the database in url exists
    // uses a maintenance database depending on driver
    fn database_exists(url: &str) -> BoxFuture<'_, Result<bool>>;

    // drop database in url
    // uses a maintenance database depending on driver
    fn drop_database(url: &str) -> BoxFuture<'_, Result<()>>;
}

// 'e = Executor
pub trait Migrate {
    // ensure migrations table exists
    // will create or migrate it if needed
    fn ensure_migrations_table(&mut self) -> BoxFuture<'_, Result<()>>;

    // Return the version on which the database is dirty or None otherwise.
    // "dirty" means there is a partially applied migration that failed.
    fn dirty_version(&mut self) -> BoxFuture<'_, Result<Option<i64>>>;

    // Return the current version and if the database is "dirty".
    // "dirty" means there is a partially applied migration that failed.
    #[deprecated]
    fn version(&mut self) -> BoxFuture<'_, Result<Option<(i64, bool)>>>;

    // validate the migration
    // checks that it does exist on the database and that the checksum matches
    #[deprecated]
    fn validate<'e: 'm, 'm>(
        &'e mut self,
        migration: &'m Migration,
    ) -> BoxFuture<'m, MigrateResult<()>>;

    // Return the ordered list of applied migrations
    fn list_applied_migrations(&mut self) -> BoxFuture<'_, Result<Vec<AppliedMigration>>>;

    // Should acquire a database lock so that only one migration process
    // can run at a time. [`Migrate`] will call this function before applying
    // any migrations.
    fn lock(&mut self) -> BoxFuture<'_, Result<()>>;

    // Should release the lock. [`Migrate`] will call this function after all
    // migrations have been run.
    fn unlock(&mut self) -> BoxFuture<'_, Result<()>>;

    // run SQL from migration in a DDL transaction
    // insert new row to [_migrations] table on completion (success or failure)
    // returns the time taking to run the migration SQL
    fn apply<'e: 'm, 'm>(&'e mut self, migration: &'m Migration)
        -> BoxFuture<'m, Result<Duration>>;

    // run a revert SQL from migration in a DDL transaction
    // deletes the row in [_migrations] table with specified migration version on completion (success or failure)
    // returns the time taking to run the migration SQL
    fn revert<'e: 'm, 'm>(
        &'e mut self,
        migration: &'m Migration,
    ) -> BoxFuture<'m, Result<Duration>>;
}
