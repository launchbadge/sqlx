use anyhow::{bail, Context};
use console::style;
use sqlx::migrate::{Migrate, MigrateError, Migrator};
use sqlx::{AnyConnection, Connection};
use std::fs::{self, File};
use std::io::Write;
use std::path::Path;

const MIGRATION_FOLDER: &'static str = "migrations";

pub fn add(description: &str, not_revertable: bool) -> anyhow::Result<()> {
    use chrono::prelude::*;
    use std::path::PathBuf;

    fs::create_dir_all(MIGRATION_FOLDER).context("Unable to create migrations directory")?;

    let date_time = Utc::now().format("%Y%m%d%H%M%S").to_string();
    let description = &description.replace(' ', "_");
    let base_file_name = format!("{}_{}", date_time, description);

    let file_names: Vec<String>;

    if not_revertable {
        file_names = vec![format!("{}.sql", base_file_name)]
    } else {
        let up = format!("{}_up.sql", base_file_name);
        let down = format!("{}_down.sql", base_file_name);
        file_names = vec![up, down]
    }
    
    for name in file_names {
        let mut path = PathBuf::new();
        path.push(MIGRATION_FOLDER);
        path.push(&name);
    
        println!("Creating {}", style(path.display()).cyan());
    
        let mut file = File::create(&path).context("Failed to create migration file")?;
    
        file.write_all(b"-- Add migration script here\n")?;
    }

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

pub async fn run(uri: &str) -> anyhow::Result<()> {
    let migrator = Migrator::new(Path::new(MIGRATION_FOLDER)).await?;
    let mut conn = AnyConnection::connect(uri).await?;

    conn.ensure_migrations_table().await?;

    let (version, dirty) = conn.version().await?.unwrap_or((0, false));

    if dirty {
        bail!(MigrateError::Dirty(version));
    }

    for migration in migrator.iter() {
        if migration.version > version {
            let elapsed = conn.apply(migration).await?;

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
