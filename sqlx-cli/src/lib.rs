use crate::opt::{Command, DatabaseCommand, MigrateCommand};
use anyhow::anyhow;
use dotenv::dotenv;
use std::env;

mod database;
// mod migration;
// mod migrator;
mod migrate;
mod opt;
mod prepare;

pub use crate::opt::Opt;

pub async fn run(opt: Opt) -> anyhow::Result<()> {
    dotenv().ok();

    let database_url = match opt.database_url {
        Some(db_url) => db_url,
        None => env::var("DATABASE_URL")
            .map_err(|_| anyhow!("The DATABASE_URL environment variable must be set"))?,
    };

    match opt.command {
        Command::Migrate(migrate) => match migrate.command {
            MigrateCommand::Add { description } => migrate::add(&description)?,
            MigrateCommand::Run => migrate::run(&database_url).await?,
            MigrateCommand::Info => migrate::info(&database_url).await?,
        },

        Command::Database(database) => match database.command {
            DatabaseCommand::Create => database::create(&database_url).await?,
            DatabaseCommand::Drop { yes } => database::drop(&database_url, !yes).await?,
        },

        Command::Prepare { check: false, args } => prepare::run(&database_url, args)?,

        Command::Prepare { check: true, args } => prepare::check(&database_url, args)?,
    };

    Ok(())
}
