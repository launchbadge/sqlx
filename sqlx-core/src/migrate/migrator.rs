use crate::acquire::Acquire;
use crate::migrate::{Migrate, MigrateError, Migration, MigrationSource};
use std::borrow::Cow;
use std::ops::Deref;
use std::slice;

#[derive(Debug)]
pub struct Migrator {
    pub migrations: Cow<'static, [Migration]>,
}

impl Migrator {
    /// Creates a new instance with the given source.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// # use sqlx_core::migrate::MigrateError;
    /// # fn main() -> Result<(), MigrateError> {
    /// # sqlx_rt::block_on(async move {
    /// # use sqlx_core::migrate::Migrator;
    /// use std::path::Path;
    ///
    /// // Read migrations from a local folder: ./migrations
    /// let m = Migrator::new(Path::new("./migrations")).await?;
    /// # Ok(())
    /// # })
    /// # }
    /// ```
    pub async fn new<'s, S>(source: S) -> Result<Self, MigrateError>
    where
        S: MigrationSource<'s>,
    {
        Ok(Self {
            migrations: Cow::Owned(source.resolve().await.map_err(MigrateError::Source)?),
        })
    }

    /// Get an iterator over all known migrations.
    pub fn iter(&self) -> slice::Iter<'_, Migration> {
        self.migrations.iter()
    }

    /// Run any pending migrations against the database; and, validate previously applied migrations
    /// against the current migration source to detect accidental changes in previously-applied migrations.
    pub async fn run<'a, A>(&self, migrator: A) -> Result<(), MigrateError>
    where
        A: Acquire<'a>,
        <A::Connection as Deref>::Target: Migrate,
    {
        let mut conn = migrator.acquire().await?;

        // lock the database for exclusive access by the migrator
        conn.lock().await?;

        // creates [_migrations] table only if needed
        // eventually this will likely migrate previous versions of the table
        conn.ensure_migrations_table().await?;

        let (version, dirty) = conn.version().await?.unwrap_or((0, false));

        if dirty {
            return Err(MigrateError::Dirty(version));
        }

        for migration in self.iter() {
            if migration.version > version {
                conn.apply(migration).await?;
            } else {
                conn.validate(migration).await?;
            }
        }

        // unlock the migrator to allow other migrators to run
        // but do nothing as we already migrated
        conn.unlock().await?;

        Ok(())
    }
}
