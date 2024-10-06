use crate::opt::ConnectOpts;
use anyhow::{bail, Context};
use chrono::Utc;
use console::style;
use sqlx::migrate::{AppliedMigration, Migrate, MigrateError, MigrationType, Migrator};
use sqlx::Connection;
use std::borrow::Cow;
use std::collections::{HashMap, HashSet};
use std::fmt::Write;
use std::fs::{self, File};
use std::path::Path;
use std::time::Duration;

fn create_file(
    migration_source: &str,
    file_prefix: &str,
    description: &str,
    migration_type: MigrationType,
) -> anyhow::Result<()> {
    use std::path::PathBuf;

    let mut file_name = file_prefix.to_string();
    file_name.push('_');
    file_name.push_str(&description.replace(' ', "_"));
    file_name.push_str(migration_type.suffix());

    let mut path = PathBuf::new();
    path.push(migration_source);
    path.push(&file_name);

    println!("Creating {}", style(path.display()).cyan());

    let mut file = File::create(&path).context("Failed to create migration file")?;

    std::io::Write::write_all(&mut file, migration_type.file_content().as_bytes())?;

    Ok(())
}

enum MigrationOrdering {
    Timestamp(String),
    Sequential(String),
}

impl MigrationOrdering {
    fn timestamp() -> MigrationOrdering {
        Self::Timestamp(Utc::now().format("%Y%m%d%H%M%S").to_string())
    }

    fn sequential(version: i64) -> MigrationOrdering {
        Self::Sequential(format!("{version:04}"))
    }

    fn file_prefix(&self) -> &str {
        match self {
            MigrationOrdering::Timestamp(prefix) => prefix,
            MigrationOrdering::Sequential(prefix) => prefix,
        }
    }

    fn infer(sequential: bool, timestamp: bool, migrator: &Migrator) -> Self {
        match (timestamp, sequential) {
            (true, true) => panic!("Impossible to specify both timestamp and sequential mode"),
            (true, false) => MigrationOrdering::timestamp(),
            (false, true) => MigrationOrdering::sequential(
                migrator
                    .iter()
                    .last()
                    .map_or(1, |last_migration| last_migration.version + 1),
            ),
            (false, false) => {
                // inferring the naming scheme
                let migrations = migrator
                    .iter()
                    .filter(|migration| migration.migration_type.is_up_migration())
                    .rev()
                    .take(2)
                    .collect::<Vec<_>>();
                if let [last, pre_last] = &migrations[..] {
                    // there are at least two migrations, compare the last twothere's only one existing migration
                    if last.version - pre_last.version == 1 {
                        // their version numbers differ by 1, infer sequential
                        MigrationOrdering::sequential(last.version + 1)
                    } else {
                        MigrationOrdering::timestamp()
                    }
                } else if let [last] = &migrations[..] {
                    // there is only one existing migration
                    if last.version == 0 || last.version == 1 {
                        // infer sequential if the version number is 0 or 1
                        MigrationOrdering::sequential(last.version + 1)
                    } else {
                        MigrationOrdering::timestamp()
                    }
                } else {
                    MigrationOrdering::timestamp()
                }
            }
        }
    }
}

pub async fn add(
    migration_source: &str,
    description: &str,
    reversible: bool,
    sequential: bool,
    timestamp: bool,
) -> anyhow::Result<()> {
    fs::create_dir_all(migration_source).context("Unable to create migrations directory")?;

    let migrator = Migrator::new(Path::new(migration_source)).await?;
    // Type of newly created migration will be the same as the first one
    // or reversible flag if this is the first migration
    let migration_type = MigrationType::infer(&migrator, reversible);

    let ordering = MigrationOrdering::infer(sequential, timestamp, &migrator);
    let file_prefix = ordering.file_prefix();

    if migration_type.is_reversible() {
        create_file(
            migration_source,
            file_prefix,
            description,
            MigrationType::ReversibleUp,
        )?;
        create_file(
            migration_source,
            file_prefix,
            description,
            MigrationType::ReversibleDown,
        )?;
    } else {
        create_file(
            migration_source,
            file_prefix,
            description,
            MigrationType::Simple,
        )?;
    }

    // if the migrations directory is empty
    let has_existing_migrations = fs::read_dir(migration_source)
        .map(|mut dir| dir.next().is_some())
        .unwrap_or(false);

    if !has_existing_migrations {
        let quoted_source = if migration_source != "migrations" {
            format!("{migration_source:?}")
        } else {
            "".to_string()
        };

        // Provide a link to the current version in case the details change.
        // Patch version is deliberately omitted.
        let version = if let (Some(major), Some(minor)) = (
            // Don't fail if we're not being built by Cargo
            option_env!("CARGO_PKG_VERSION_MAJOR"),
            option_env!("CARGO_PKG_VERSION_MINOR"),
        ) {
            format!("{major}.{minor}")
        } else {
            // If a version isn't available, "latest" is fine.
            "latest".to_string()
        };

        print!(
            r#"
Congratulations on creating your first migration!

Did you know you can embed your migrations in your application binary?
On startup, after creating your database connection or pool, add:

sqlx::migrate!({quoted_source}).run(<&your_pool OR &mut your_connection>).await?;

Note that the compiler won't pick up new migrations if no Rust source files have changed.
You can create a Cargo build script to work around this with `sqlx migrate build-script`.

See: https://docs.rs/sqlx/{version}/sqlx/macro.migrate.html
"#,
        );
    }

    Ok(())
}

fn short_checksum(checksum: &[u8]) -> String {
    let mut s = String::with_capacity(checksum.len() * 2);
    for b in checksum {
        write!(&mut s, "{b:02x?}").expect("should not fail to write to str");
    }
    s
}

pub async fn info(migration_source: &str, connect_opts: &ConnectOpts) -> anyhow::Result<()> {
    let migrator = Migrator::new(Path::new(migration_source)).await?;
    let mut conn = crate::connect(connect_opts).await?;

    conn.ensure_migrations_table().await?;

    let applied_migrations: HashMap<_, _> = conn
        .list_applied_migrations()
        .await?
        .into_iter()
        .map(|m| (m.version, m))
        .collect();

    for migration in migrator.iter() {
        if migration.migration_type.is_down_migration() {
            // Skipping down migrations
            continue;
        }

        let applied = applied_migrations.get(&migration.version);

        let (status_msg, mismatched_checksum) = if let Some(applied) = applied {
            if applied.checksum != migration.checksum {
                (style("installed (different checksum)").red(), true)
            } else {
                (style("installed").green(), false)
            }
        } else {
            (style("pending").yellow(), false)
        };

        println!(
            "{}/{} {}",
            style(migration.version).cyan(),
            status_msg,
            migration.description
        );

        if mismatched_checksum {
            println!(
                "applied migration had checksum {}",
                short_checksum(
                    &applied
                        .map(|a| a.checksum.clone())
                        .unwrap_or_else(|| Cow::Owned(vec![]))
                ),
            );
            println!(
                "local migration has checksum {}",
                short_checksum(&migration.checksum)
            )
        }
    }

    let _ = conn.close().await;

    Ok(())
}

fn validate_applied_migrations(
    applied_migrations: &[AppliedMigration],
    migrator: &Migrator,
    ignore_missing: bool,
) -> Result<(), MigrateError> {
    if ignore_missing {
        return Ok(());
    }

    let migrations: HashSet<_> = migrator.iter().map(|m| m.version).collect();

    for applied_migration in applied_migrations {
        if !migrations.contains(&applied_migration.version) {
            return Err(MigrateError::VersionMissing(applied_migration.version));
        }
    }

    Ok(())
}

pub async fn run(
    migration_source: &str,
    connect_opts: &ConnectOpts,
    dry_run: bool,
    ignore_missing: bool,
    target_version: Option<i64>,
) -> anyhow::Result<()> {
    let migrator = Migrator::new(Path::new(migration_source)).await?;
    if let Some(target_version) = target_version {
        if !migrator.version_exists(target_version) {
            bail!(MigrateError::VersionNotPresent(target_version));
        }
    }

    let mut conn = crate::connect(connect_opts).await?;

    conn.ensure_migrations_table().await?;

    let version = conn.dirty_version().await?;
    if let Some(version) = version {
        bail!(MigrateError::Dirty(version));
    }

    let applied_migrations = conn.list_applied_migrations().await?;
    validate_applied_migrations(&applied_migrations, &migrator, ignore_missing)?;

    let latest_version = applied_migrations
        .iter()
        .max_by(|x, y| x.version.cmp(&y.version))
        .map(|migration| migration.version)
        .unwrap_or(0);
    if let Some(target_version) = target_version {
        if target_version < latest_version {
            bail!(MigrateError::VersionTooOld(target_version, latest_version));
        }
    }

    let applied_migrations: HashMap<_, _> = applied_migrations
        .into_iter()
        .map(|m| (m.version, m))
        .collect();

    for migration in migrator.iter() {
        if migration.migration_type.is_down_migration() {
            // Skipping down migrations
            continue;
        }

        match applied_migrations.get(&migration.version) {
            Some(applied_migration) => {
                if migration.checksum != applied_migration.checksum {
                    bail!(MigrateError::VersionMismatch(migration.version));
                }
            }
            None => {
                let skip =
                    target_version.is_some_and(|target_version| migration.version > target_version);

                let elapsed = if dry_run || skip {
                    Duration::new(0, 0)
                } else {
                    conn.apply(migration).await?
                };
                let text = if skip {
                    "Skipped"
                } else if dry_run {
                    "Can apply"
                } else {
                    "Applied"
                };

                println!(
                    "{} {}/{} {} {}",
                    text,
                    style(migration.version).cyan(),
                    style(migration.migration_type.label()).green(),
                    migration.description,
                    style(format!("({elapsed:?})")).dim()
                );
            }
        }
    }

    // Close the connection before exiting:
    // * For MySQL and Postgres this should ensure timely cleanup on the server side,
    //   including decrementing the open connection count.
    // * For SQLite this should checkpoint and delete the WAL file to ensure the migrations
    //   were actually applied to the database file and aren't just sitting in the WAL file.
    let _ = conn.close().await;

    Ok(())
}

pub async fn revert(
    migration_source: &str,
    connect_opts: &ConnectOpts,
    dry_run: bool,
    ignore_missing: bool,
    target_version: Option<i64>,
) -> anyhow::Result<()> {
    let migrator = Migrator::new(Path::new(migration_source)).await?;
    if let Some(target_version) = target_version {
        if target_version != 0 && !migrator.version_exists(target_version) {
            bail!(MigrateError::VersionNotPresent(target_version));
        }
    }

    let mut conn = crate::connect(connect_opts).await?;

    conn.ensure_migrations_table().await?;

    let version = conn.dirty_version().await?;
    if let Some(version) = version {
        bail!(MigrateError::Dirty(version));
    }

    let applied_migrations = conn.list_applied_migrations().await?;
    validate_applied_migrations(&applied_migrations, &migrator, ignore_missing)?;

    let latest_version = applied_migrations
        .iter()
        .max_by(|x, y| x.version.cmp(&y.version))
        .map(|migration| migration.version)
        .unwrap_or(0);
    if let Some(target_version) = target_version {
        if target_version > latest_version {
            bail!(MigrateError::VersionTooNew(target_version, latest_version));
        }
    }

    let applied_migrations: HashMap<_, _> = applied_migrations
        .into_iter()
        .map(|m| (m.version, m))
        .collect();

    let mut is_applied = false;
    for migration in migrator.iter().rev() {
        if !migration.migration_type.is_down_migration() {
            // Skipping non down migration
            // This will skip any simple or up migration file
            continue;
        }

        if applied_migrations.contains_key(&migration.version) {
            let skip =
                target_version.is_some_and(|target_version| migration.version <= target_version);

            let elapsed = if dry_run || skip {
                Duration::new(0, 0)
            } else {
                conn.revert(migration).await?
            };
            let text = if skip {
                "Skipped"
            } else if dry_run {
                "Can apply"
            } else {
                "Applied"
            };

            println!(
                "{} {}/{} {} {}",
                text,
                style(migration.version).cyan(),
                style(migration.migration_type.label()).green(),
                migration.description,
                style(format!("({elapsed:?})")).dim()
            );

            is_applied = true;

            // Only a single migration will be reverted at a time if no target
            // version is supplied, so we break.
            if target_version.is_none() {
                break;
            }
        }
    }
    if !is_applied {
        println!("No migrations available to revert");
    }

    let _ = conn.close().await;

    Ok(())
}

pub fn build_script(migration_source: &str, force: bool) -> anyhow::Result<()> {
    anyhow::ensure!(
        Path::new("Cargo.toml").exists(),
        "must be run in a Cargo project root"
    );

    anyhow::ensure!(
        (force || !Path::new("build.rs").exists()),
        "build.rs already exists; use --force to overwrite"
    );

    let contents = format!(
        r#"// generated by `sqlx migrate build-script`
fn main() {{
    // trigger recompilation when a new migration is added
    println!("cargo:rerun-if-changed={migration_source}");
}}
"#,
    );

    fs::write("build.rs", contents)?;

    println!("Created `build.rs`; be sure to check it into version control!");

    Ok(())
}
