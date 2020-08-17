use anyhow::{bail, Context};
use console::style;
use sqlx::migrate::{Migrate, MigrateError, Migrator};
use sqlx::{AnyConnection, Connection};
use std::fs::{self, File};
use std::io::Write;
use std::path::Path;

const MIGRATION_FOLDER: &'static str = "migrations";

pub fn add(description: &str) -> anyhow::Result<()> {
    use chrono::prelude::*;
    use std::path::PathBuf;

    fs::create_dir_all(MIGRATION_FOLDER).context("Unable to create migrations directory")?;

    let dt = Utc::now();
    let mut file_name = dt.format("%Y%m%d%H%M%S").to_string();
    file_name.push_str("_");
    file_name.push_str(&description.replace(' ', "_"));
    file_name.push_str(".sql");

    let mut path = PathBuf::new();
    path.push(MIGRATION_FOLDER);
    path.push(&file_name);

    println!("Creating {}", style(path.display()).cyan());

    let mut file = File::create(&path).context("Failed to create migration file")?;

    file.write_all(b"-- Add migration script here\n")?;

    Ok(())
}

pub async fn info(uri: &str) -> anyhow::Result<()> {
    let migrator = Migrator::new(Path::new(MIGRATION_FOLDER)).await?;
    let mut conn = AnyConnection::connect(uri).await?;

    conn.ensure_migrations_table().await?;

    let (version, _) = conn.version().await?.unwrap_or((0, false));

    for migration in migrator.iter() {
        println!(
            "{}/{} {}",
            style(migration.version).cyan(),
            if version >= migration.version {
                style("installed").green()
            } else {
                style("pending").yellow()
            },
            migration.description,
        );
    }

    Ok(())
}

pub async fn run(uri: &str, fake: bool) -> anyhow::Result<()> {
    let migrator = Migrator::new(Path::new(MIGRATION_FOLDER)).await?;
    let mut conn = AnyConnection::connect(uri).await?;

    conn.ensure_migrations_table().await?;

    let (version, dirty) = conn.version().await?.unwrap_or((0, false));

    if dirty {
        bail!(MigrateError::Dirty(version));
    }

    for migration in migrator.iter() {
        if migration.version > version {
            let elapsed = conn.apply(migration, fake).await?;

            println!(
                "{}/{} {} {}",
                style(migration.version).cyan(),
                style("migrate").green(),
                migration.description,
                style(format!("({:?})", elapsed)).dim()
            );
        } else {
            conn.validate(migration).await?;
        }
    }

    Ok(())
}
