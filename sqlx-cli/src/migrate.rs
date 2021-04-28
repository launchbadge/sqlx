use anyhow::{bail, Context};
use chrono::Utc;
use console::style;
use sqlx::migrate::{AppliedMigration, Migrate, MigrateError, MigrationType, Migrator};
use sqlx::{AnyConnection, Connection};
use std::collections::{HashMap, HashSet};
use std::fs::{self, File};
use std::io::Write;
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
    file_name.push_str("_");
    file_name.push_str(&description.replace(' ', "_"));
    file_name.push_str(migration_type.suffix());

    let mut path = PathBuf::new();
    path.push(migration_source);
    path.push(&file_name);

    println!("Creating {}", style(path.display()).cyan());

    let mut file = File::create(&path).context("Failed to create migration file")?;

    file.write_all(migration_type.file_content().as_bytes())?;

    Ok(())
}

pub async fn add(
    migration_source: &str,
    description: &str,
    reversible: bool,
) -> anyhow::Result<()> {
    fs::create_dir_all(migration_source).context("Unable to create migrations directory")?;

    let migrator = Migrator::new(Path::new(migration_source)).await?;
    // This checks if all existing migrations are of the same type as the reverisble flag passed
    for migration in migrator.iter() {
        if migration.migration_type.is_reversible() != reversible {
            bail!(MigrateError::InvalidMixReversibleAndSimple);
        }
    }

    let dt = Utc::now();
    let file_prefix = dt.format("%Y%m%d%H%M%S").to_string();
    if reversible {
        create_file(
            migration_source,
            &file_prefix,
            description,
            MigrationType::ReversibleUp,
        )?;
        create_file(
            migration_source,
            &file_prefix,
            description,
            MigrationType::ReversibleDown,
        )?;
    } else {
        create_file(
            migration_source,
            &file_prefix,
            description,
            MigrationType::Simple,
        )?;
    }

    Ok(())
}

pub async fn info(migration_source: &str, uri: &str) -> anyhow::Result<()> {
    let migrator = Migrator::new(Path::new(migration_source)).await?;
    let mut conn = AnyConnection::connect(uri).await?;

    conn.ensure_migrations_table().await?;

    let applied_migrations: HashMap<_, _> = conn
        .list_applied_migrations()
        .await?
        .into_iter()
        .map(|m| (m.version, m))
        .collect();

    for migration in migrator.iter() {
        println!(
            "{}/{} {}",
            style(migration.version).cyan(),
            if applied_migrations.contains_key(&migration.version) {
                style("installed").green()
            } else {
                style("pending").yellow()
            },
            migration.description,
        );
    }

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
    uri: &str,
    dry_run: bool,
    ignore_missing: bool,
) -> anyhow::Result<()> {
    let migrator = Migrator::new(Path::new(migration_source)).await?;
    let mut conn = AnyConnection::connect(uri).await?;

    conn.ensure_migrations_table().await?;

    let version = conn.dirty_version().await?;
    if let Some(version) = version {
        bail!(MigrateError::Dirty(version));
    }

    let applied_migrations = conn.list_applied_migrations().await?;
    validate_applied_migrations(&applied_migrations, &migrator, ignore_missing)?;

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
                let elapsed = if dry_run {
                    Duration::new(0, 0)
                } else {
                    conn.apply(migration).await?
                };
                let text = if dry_run { "Can apply" } else { "Applied" };

                println!(
                    "{} {}/{} {} {}",
                    text,
                    style(migration.version).cyan(),
                    style(migration.migration_type.label()).green(),
                    migration.description,
                    style(format!("({:?})", elapsed)).dim()
                );
            }
        }
    }

    Ok(())
}

pub async fn revert(
    migration_source: &str,
    uri: &str,
    dry_run: bool,
    ignore_missing: bool,
) -> anyhow::Result<()> {
    let migrator = Migrator::new(Path::new(migration_source)).await?;
    let mut conn = AnyConnection::connect(uri).await?;

    conn.ensure_migrations_table().await?;

    let version = conn.dirty_version().await?;
    if let Some(version) = version {
        bail!(MigrateError::Dirty(version));
    }

    let applied_migrations = conn.list_applied_migrations().await?;
    validate_applied_migrations(&applied_migrations, &migrator, ignore_missing)?;

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
            let elapsed = if dry_run {
                Duration::new(0, 0)
            } else {
                conn.revert(migration).await?
            };
            let text = if dry_run { "Can apply" } else { "Applied" };

            println!(
                "{} {}/{} {} {}",
                text,
                style(migration.version).cyan(),
                style(migration.migration_type.label()).green(),
                migration.description,
                style(format!("({:?})", elapsed)).dim()
            );

            is_applied = true;
            // Only a single migration will be reverted at a time, so we break
            break;
        }
    }
    if !is_applied {
        println!("No migrations available to revert");
    }

    Ok(())
}
