use anyhow::{bail, Context};
use console::style;
use std::fs::{self, File};
use std::io::{Read, Write};

const MIGRATION_FOLDER: &str = "migrations";

pub struct Migration {
    pub name: String,
    pub sql: String,
}

pub fn add_file(name: &str) -> anyhow::Result<()> {
    use chrono::prelude::*;
    use std::path::PathBuf;

    fs::create_dir_all(MIGRATION_FOLDER).context("Unable to create migrations directory")?;

    let dt = Utc::now();
    let mut file_name = dt.format("%Y-%m-%d_%H-%M-%S").to_string();
    file_name.push_str("_");
    file_name.push_str(name);
    file_name.push_str(".sql");

    let mut path = PathBuf::new();
    path.push(MIGRATION_FOLDER);
    path.push(&file_name);

    let mut file = File::create(path).context("Failed to create file")?;
    file.write_all(b"-- Add migration script here")
        .context("Could not write to file")?;

    println!("Created migration: '{file_name}'");
    Ok(())
}

pub async fn run() -> anyhow::Result<()> {
    let migrator = crate::migrator::get()?;

    if !migrator.can_migrate_database() {
        bail!(
            "Database migrations not supported for {}",
            migrator.database_type()
        );
    }

    migrator.create_migration_table().await?;

    let migrations = load_migrations()?;

    for mig in migrations.iter() {
        let mut tx = migrator.begin_migration().await?;

        if tx.check_if_applied(&mig.name).await? {
            println!("Already applied migration: '{}'", mig.name);
            continue;
        }
        println!("Applying migration: '{}'", mig.name);

        tx.execute_migration(&mig.sql)
            .await
            .with_context(|| format!("Failed to run migration {:?}", &mig.name))?;

        tx.save_applied_migration(&mig.name)
            .await
            .context("Failed to insert migration")?;

        tx.commit().await.context("Failed")?;
    }

    Ok(())
}

pub async fn list() -> anyhow::Result<()> {
    let migrator = crate::migrator::get()?;

    if !migrator.can_migrate_database() {
        bail!(
            "Database migrations not supported for {}",
            migrator.database_type()
        );
    }

    let file_migrations = load_migrations()?;

    if migrator
        .check_if_database_exists(&migrator.get_database_name()?)
        .await?
    {
        let applied_migrations = migrator.get_migrations().await.unwrap_or_else(|_| {
            println!("Could not retrieve data from migration table");
            Vec::new()
        });

        let mut width = 0;
        for mig in file_migrations.iter() {
            width = std::cmp::max(width, mig.name.len());
        }
        for mig in file_migrations.iter() {
            let status = if applied_migrations
                .iter()
                .find(|&m| mig.name == *m)
                .is_some()
            {
                style("Applied").green()
            } else {
                style("Not Applied").yellow()
            };

            println!("{:width$}\t{}", mig.name, status, width = width);
        }

        let orphans = check_for_orphans(file_migrations, applied_migrations);

        if let Some(orphans) = orphans {
            println!("\nFound migrations applied in the database that does not have a corresponding migration file:");
            for name in orphans {
                println!("{:width$}\t{}", name, style("Orphan").red(), width = width);
            }
        }
    } else {
        println!("No database found, listing migrations");

        for mig in file_migrations {
            println!("{}", mig.name);
        }
    }

    Ok(())
}

fn load_migrations() -> anyhow::Result<Vec<Migration>> {
    let entries = fs::read_dir(&MIGRATION_FOLDER).context("Could not find 'migrations' dir")?;

    let mut migrations = Vec::new();

    for e in entries {
        if let Ok(e) = e {
            if let Ok(meta) = e.metadata() {
                if !meta.is_file() {
                    continue;
                }

                if let Some(ext) = e.path().extension() {
                    if ext != "sql" {
                        println!("Wrong ext: {ext:?}");
                        continue;
                    }
                } else {
                    continue;
                }

                let mut file = File::open(e.path())
                    .with_context(|| format!("Failed to open: '{:?}'", e.file_name()))?;
                let mut contents = String::new();
                file.read_to_string(&mut contents)
                    .with_context(|| format!("Failed to read: '{:?}'", e.file_name()))?;

                migrations.push(Migration {
                    name: e.file_name().to_str().unwrap().to_string(),
                    sql: contents,
                });
            }
        }
    }

    migrations.sort_by(|a, b| a.name.partial_cmp(&b.name).unwrap());

    Ok(migrations)
}

fn check_for_orphans(
    file_migrations: Vec<Migration>,
    applied_migrations: Vec<String>,
) -> Option<Vec<String>> {
    let orphans: Vec<String> = applied_migrations
        .iter()
        .filter(|m| !file_migrations.iter().any(|fm| fm.name == **m))
        .cloned()
        .collect();

    if orphans.len() > 0 {
        Some(orphans)
    } else {
        None
    }
}
