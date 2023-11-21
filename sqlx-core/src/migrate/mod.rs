mod error;
#[allow(clippy::module_inception)]
mod migrate;
mod migration;
mod migration_type;
mod migrator;
mod source;

pub use error::MigrateError;
pub use migrate::{Migrate, MigrateDatabase};
pub use migration::{AppliedMigration, Migration};
pub use migration_type::MigrationType;
pub use migrator::Migrator;
pub use source::MigrationSource;

pub const DEFAULT_MIGRATION_TABLE: &str = "_sqlx_migrations";
