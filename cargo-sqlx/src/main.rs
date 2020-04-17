use std::env;
use std::fs;
use std::fs::File;
use std::io::prelude::*;
use url::Url;

use dotenv::dotenv;

use structopt::StructOpt;

use anyhow::{anyhow, Context, Result};

mod database_migrator;
mod postgres;
mod sqlite;

use database_migrator::DatabaseMigrator;
use postgres::Postgres;
use sqlite::Sqlite;

const MIGRATION_FOLDER: &'static str = "migrations";

/// Sqlx commandline tool
#[derive(StructOpt, Debug)]
#[structopt(name = "Sqlx")]
enum Opt {
    Migrate(MigrationCommand),

    #[structopt(alias = "db")]
    Database(DatabaseCommand),
}

/// Adds and runs migrations
#[derive(StructOpt, Debug)]
#[structopt(name = "Sqlx migrator")]
enum MigrationCommand {
    /// Add new migration with name <timestamp>_<migration_name>.sql
    Add { name: String },

    /// Run all migrations
    Run,
}

/// Create or drops database depending on your connection string. Alias: db
#[derive(StructOpt, Debug)]
#[structopt(name = "Sqlx migrator")]
enum DatabaseCommand {
    /// Create database in url
    Create,

    /// Drop database in url
    Drop,
}

#[tokio::main]
async fn main() -> Result<()> {
    dotenv().ok();

    let db_url_raw = env::var("DATABASE_URL").context("Failed to find 'DATABASE_URL'")?;

    let db_url = Url::parse(&db_url_raw)?;

    // This code is taken from: https://github.com/launchbadge/sqlx/blob/master/sqlx-macros/src/lib.rs#L63
    match db_url.scheme() {
        #[cfg(feature = "sqlite")]
        "sqlite" => run_command(&Sqlite::new(db_url_raw )).await?,
        #[cfg(not(feature = "sqlite"))]
        "sqlite" => return Err(anyhow!("Not implemented. DATABASE_URL {} has the scheme of a SQLite database but the `sqlite` feature of sqlx was not enabled",
                            db_url)),

        #[cfg(feature = "postgres")]
        "postgresql" | "postgres" => run_command(&Postgres::new(db_url_raw)).await?,
        #[cfg(not(feature = "postgres"))]
        "postgresql" | "postgres" => Err(anyhow!("DATABASE_URL {} has the scheme of a Postgres database but the `postgres` feature of sqlx was not enabled",
                db_url)),

        #[cfg(feature = "mysql")]
        "mysql" | "mariadb" => return Err(anyhow!("Not implemented")),
        #[cfg(not(feature = "mysql"))]
        "mysql" | "mariadb" => return Err(anyhow!(
            "DATABASE_URL {} has the scheme of a MySQL/MariaDB database but the `mysql` feature of sqlx was not enabled",
             db_url
        )),

        scheme => return Err(anyhow!("unexpected scheme {:?} in DATABASE_URL {}", scheme, db_url)),
    }

    println!("All done!");
    Ok(())
}

async fn run_command(migrator: &dyn DatabaseMigrator) -> Result<()> {
    let opt = Opt::from_args();

    match opt {
        Opt::Migrate(command) => match command {
            MigrationCommand::Add { name } => add_migration_file(&name)?,
            MigrationCommand::Run => run_migrations(migrator).await?,
        },
        Opt::Database(command) => match command {
            DatabaseCommand::Create => run_create_database(migrator).await?,
            DatabaseCommand::Drop => run_drop_database(migrator).await?,
        },
    };

    Ok(())
}

async fn run_create_database(migrator: &dyn DatabaseMigrator) -> Result<()> {
    if !migrator.can_create_database() {
        return Err(anyhow!(
            "Database creation is not implemented for {}",
            migrator.database_type()
        ));
    }

    let db_name = migrator.get_database_name()?;
    let db_exists = migrator.check_if_database_exists(&db_name).await?;

    if !db_exists {
        println!("Creating database: {}", db_name);
        Ok(migrator.create_database(&db_name).await?)
    } else {
        println!("Database already exists, aborting");
        Ok(())
    }
}

async fn run_drop_database(migrator: &dyn DatabaseMigrator) -> Result<()> {
    use std::io;

    if !migrator.can_drop_database() {
        return Err(anyhow!(
            "Database drop is not implemented for {}",
            migrator.database_type()
        ));
    }

    let db_name = migrator.get_database_name()?;
    let db_exists = migrator.check_if_database_exists(&db_name).await?;

    if db_exists {

        loop {
            println!("\nAre you sure you want to drop the database: {}? Y/n", db_name);
    
            let mut input = String::new();
        
            io::stdin()
                .read_line(&mut input)
                .context("Failed to read line")?;
        
            match input.trim() {
                "Y" => break,
                "N" => return Ok(()),
                "n" => return Ok(()),
                _ => continue,
            };
        };

        println!("Dropping database: {}", db_name);
        Ok(migrator.drop_database(&db_name).await?)
    } else {
        println!("Database does not exists, aborting");
        Ok(())
    }
}

fn add_migration_file(name: &str) -> Result<()> {
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

    println!("Created migration: '{}'", file_name);
    Ok(())
}

pub struct Migration {
    pub name: String,
    pub sql: String,
}

fn load_migrations() -> Result<Vec<Migration>> {
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
                        println!("Wrong ext: {:?}", ext);
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

async fn run_migrations(migrator: &dyn DatabaseMigrator) -> Result<()> {
    if !migrator.can_migrate_database() {
        return Err(anyhow!(
            "Database migrations not implemented for {}",
            migrator.database_type()
        ));
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
