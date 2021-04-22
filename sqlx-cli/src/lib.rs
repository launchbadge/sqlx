use crate::opt::{Command, DatabaseCommand, MigrateCommand};
use anyhow::{anyhow, Context};
use dotenv::dotenv;
use prepare::PrepareCtx;
use std::env;

mod cargo;
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
                database::reset(&source, &database_url, !yes).await?
            }
            DatabaseCommand::Setup { source } => database::setup(&source, &database_url).await?,
        },

        Command::Prepare {
            check,
            workspace,
            args,
        } => {
            let cargo_path = cargo::cargo_path()?;
            let manifest_dir = cargo::manifest_dir()?;
            let metadata = cargo::metadata(&cargo_path)
                .context("`prepare` subcommand may only be invoked as `cargo sqlx prepare`")?;

            let ctx = PrepareCtx {
                workspace,
                cargo: cargo_path,
                cargo_args: args,
                manifest_dir,
                target_dir: metadata.target_directory,
                workspace_root: metadata.workspace_root,
            };

            if check {
                prepare::check(&ctx)?
            } else {
                prepare::run(&ctx)?
            }
        }
    };

    Ok(())
}
