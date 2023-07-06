use crate::error::BoxDynError;
use crate::fs;
use crate::migrate::{Migration, MigrationType};
use futures_core::future::BoxFuture;

use std::borrow::Cow;
use std::fmt::Debug;
use std::path::{Path, PathBuf};

/// In the default implementation, a MigrationSource is a directory which
/// contains the migration SQL scripts. All these scripts must be stored in
/// files with names using the format `<VERSION>_<DESCRIPTION>.sql`, where
/// `<VERSION>` is a string that can be parsed into `i64` and its value is
/// greater than zero, and `<DESCRIPTION>` is a string.
///
/// Files that don't match this format are silently ignored.
///
/// You can create a new empty migration script using sqlx-cli:
/// `sqlx migrate add <DESCRIPTION>`.
///
/// Note that migrations for each database are tracked using the
/// `_sqlx_migrations` table (stored in the database). If a migration's hash
/// changes and it has already been run, this will cause an error.
pub trait MigrationSource<'s>: Debug {
    fn resolve(self) -> BoxFuture<'s, Result<Vec<Migration>, BoxDynError>>;
}

impl<'s> MigrationSource<'s> for &'s Path {
    fn resolve(self) -> BoxFuture<'s, Result<Vec<Migration>, BoxDynError>> {
        Box::pin(async move {
            let mut s = fs::read_dir(self.canonicalize()?).await?;
            let mut migrations = Vec::new();

            while let Some(entry) = s.next().await? {
                // std::fs::metadata traverses symlinks
                if !std::fs::metadata(&entry.path)?.is_file() {
                    // not a file; ignore
                    continue;
                }

                let file_name = entry.file_name.to_string_lossy();

                let parts = file_name.splitn(2, '_').collect::<Vec<_>>();

                if parts.len() != 2 || !parts[1].ends_with(".sql") {
                    // not of the format: <VERSION>_<DESCRIPTION>.sql; ignore
                    continue;
                }

                let version: i64 = parts[0].parse()?;

                let migration_type = MigrationType::from_filename(parts[1]);
                // remove the `.sql` and replace `_` with ` `
                let description = parts[1]
                    .trim_end_matches(migration_type.suffix())
                    .replace('_', " ")
                    .to_owned();

                let sql = fs::read_to_string(&entry.path).await?;

                migrations.push(Migration::new(
                    version,
                    Cow::Owned(description),
                    migration_type,
                    Cow::Owned(sql),
                ));
            }

            // ensure that we are sorted by `VERSION ASC`
            migrations.sort_by_key(|m| m.version);

            Ok(migrations)
        })
    }
}

impl MigrationSource<'static> for PathBuf {
    fn resolve(self) -> BoxFuture<'static, Result<Vec<Migration>, BoxDynError>> {
        Box::pin(async move { self.as_path().resolve().await })
    }
}
