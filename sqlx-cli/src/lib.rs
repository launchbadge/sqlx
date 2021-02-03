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
            MigrateCommand::Add {
                description,
                reversible,
            } => migrate::add(&migrate.source, &description, reversible).await?,
            MigrateCommand::Run {
                dry_run,
                ignore_missing,
            } => migrate::run(&migrate.source, &database_url, dry_run, ignore_missing).await?,
            MigrateCommand::Revert {
                dry_run,
                ignore_missing,
            } => migrate::revert(&migrate.source, &database_url, dry_run, ignore_missing).await?,
            MigrateCommand::Info => migrate::info(&migrate.source, &database_url).await?,
        },

        Command::Database(database) => match database.command {
            DatabaseCommand::Create => database::create(&database_url).await?,
            DatabaseCommand::Drop { yes } => database::drop(&database_url, !yes).await?,
            DatabaseCommand::Reset { yes, source } => {
                database::reset(&source, &database_url, yes).await?
            }
            DatabaseCommand::Setup { source } => database::setup(&source, &database_url).await?,
        },

        Command::Prepare {
            check: false,
            merged,
            args,
        } => prepare::run(&database_url, merged, args)?,

        Command::Prepare {
            check: true,
            merged,
            args,
        } => prepare::check(&database_url, merged, args)?,
    };

    Ok(())
}
