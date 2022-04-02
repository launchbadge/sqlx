use anyhow::Result;

use crate::opt::{Command, DatabaseCommand, MigrateCommand};

mod database;
// mod migration;
// mod migrator;
mod migrate;
mod opt;
mod prepare;

pub use crate::opt::Opt;

pub async fn run(opt: Opt) -> Result<()> {
    match opt.command {
        Command::Migrate(migrate) => match migrate.command {
            MigrateCommand::Add {
                source,
                description,
                reversible,
            } => migrate::add(source.resolve(&migrate.source), &description, reversible).await?,
            MigrateCommand::Run {
                source,
                dry_run,
                ignore_missing,
                database_url,
            } => {
                migrate::run(
                    source.resolve(&migrate.source),
                    &database_url,
                    dry_run,
                    *ignore_missing,
                )
                .await?
            }
            MigrateCommand::Revert {
                source,
                dry_run,
                ignore_missing,
                database_url,
            } => {
                migrate::revert(
                    source.resolve(&migrate.source),
                    &database_url,
                    dry_run,
                    *ignore_missing,
                )
                .await?
            }
            MigrateCommand::Info {
                source,
                database_url,
            } => migrate::info(source.resolve(&migrate.source), &database_url).await?,
            MigrateCommand::BuildScript { source, force } => {
                migrate::build_script(source.resolve(&migrate.source), force)?
            }
        },

        Command::Database(database) => match database.command {
            DatabaseCommand::Create { database_url } => database::create(&database_url).await?,
            DatabaseCommand::Drop {
                confirmation,
                database_url,
            } => database::drop(&database_url, !confirmation).await?,
            DatabaseCommand::Reset {
                confirmation,
                source,
                database_url,
            } => database::reset(&source, &database_url, !confirmation).await?,
            DatabaseCommand::Setup {
                source,
                database_url,
            } => database::setup(&source, &database_url).await?,
        },

        Command::Prepare {
            check: false,
            merged,
            args,
            database_url,
        } => prepare::run(&database_url, merged, args)?,

        Command::Prepare {
            check: true,
            merged,
            args,
            database_url,
        } => prepare::check(&database_url, merged, args)?,
    };

    Ok(())
}
