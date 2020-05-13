use std::env;
use std::fs;
use std::fs::File;
use std::io::prelude::*;
use url::Url;

use dotenv::dotenv;

use structopt::StructOpt;

use anyhow::{anyhow, Context};
use console::style;
use dialoguer::Confirmation;

mod migrator;

mod db;
mod migration;
mod prepare;

use migrator::DatabaseMigrator;

/// Sqlx commandline tool
#[derive(StructOpt, Debug)]
#[structopt(name = "Sqlx")]
enum Opt {
    #[structopt(alias = "mig")]
    Migrate(MigrationCommand),

    #[structopt(alias = "db")]
    Database(DatabaseCommand),

    /// Enables offline mode for a project utilizing `query!()` and related macros.
    /// May only be run as `cargo sqlx prepare`.
    ///
    /// Saves data for all invocations of `query!()` and friends in the project so that it may be
    /// built in offline mode, i.e. so compilation does not require connecting to a running database.
    /// Outputs to `sqlx-data.json` in the current directory.
    ///
    /// Offline mode can be activated simply by removing `DATABASE_URL` from the environment or
    /// building without a `.env` file.
    #[structopt(alias = "prep")]
    Prepare {
        /// If this flag is passed, instead of overwriting `sqlx-data.json` in the current directory,
        /// that file is loaded and compared against the current output of the prepare step; if
        /// there is a mismatch, an error is reported and the process exits with a nonzero exit code.
        ///
        /// Intended for use in CI.
        #[structopt(long)]
        check: bool,
    },
}

/// Adds and runs migrations. Alias: mig
#[derive(StructOpt, Debug)]
#[structopt(name = "Sqlx migrator")]
enum MigrationCommand {
    /// Add new migration with name <timestamp>_<migration_name>.sql
    Add { name: String },

    /// Run all migrations
    Run,

    /// List all migrations
    List,
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
async fn main() -> anyhow::Result<()> {
    dotenv().ok();

    let opt = Opt::from_args();

    match opt {
        Opt::Migrate(command) => match command {
            MigrationCommand::Add { name } => migration::add_file(&name)?,
            MigrationCommand::Run => migration::run().await?,
            MigrationCommand::List => migration::list().await?,
        },
        Opt::Database(command) => match command {
            DatabaseCommand::Create => db::run_create().await?,
            DatabaseCommand::Drop => db::run_drop().await?,
        },
        Opt::Prepare { check: false } => prepare::run()?,
        Opt::Prepare { check: true } => prepare::check()?,
    };

    println!("All done!");
    Ok(())
}
