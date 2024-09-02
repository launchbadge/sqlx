use std::io;
use std::time::Duration;

use anyhow::Result;
use backon::{ExponentialBuilder, RetryableWithContext};
use futures::Future;

use sqlx::{AnyConnection, Connection};

use crate::opt::{Command, ConnectOpts, DatabaseCommand, MigrateCommand};

mod database;
mod metadata;
// mod migration;
// mod migrator;
#[cfg(feature = "completions")]
mod completions;
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
                sequential,
                timestamp,
            } => migrate::add(&source, &description, reversible, sequential, timestamp).await?,
            MigrateCommand::Run {
                source,
                dry_run,
                ignore_missing,
                connect_opts,
                target_version,
            } => {
                migrate::run(
                    &source,
                    &connect_opts,
                    dry_run,
                    *ignore_missing,
                    target_version,
                )
                .await?
            }
            MigrateCommand::Revert {
                source,
                dry_run,
                ignore_missing,
                connect_opts,
                target_version,
            } => {
                migrate::revert(
                    &source,
                    &connect_opts,
                    dry_run,
                    *ignore_missing,
                    target_version,
                )
                .await?
            }
            MigrateCommand::Info {
                source,
                connect_opts,
            } => migrate::info(&source, &connect_opts).await?,
            MigrateCommand::BuildScript { source, force } => migrate::build_script(&source, force)?,
        },

        Command::Database(database) => match database.command {
            DatabaseCommand::Create { connect_opts } => database::create(&connect_opts).await?,
            DatabaseCommand::Drop {
                confirmation,
                connect_opts,
                force,
            } => database::drop(&connect_opts, !confirmation.yes, force).await?,
            DatabaseCommand::Reset {
                confirmation,
                source,
                connect_opts,
                force,
            } => database::reset(&source, &connect_opts, !confirmation.yes, force).await?,
            DatabaseCommand::Setup {
                source,
                connect_opts,
            } => database::setup(&source, &connect_opts).await?,
        },

        Command::Prepare {
            check,
            all,
            workspace,
            connect_opts,
            args,
        } => prepare::run(check, all, workspace, connect_opts, args).await?,

        #[cfg(feature = "completions")]
        Command::Completions { shell } => completions::run(shell),
    };

    Ok(())
}

/// Attempt to connect to the database server, retrying up to `ops.connect_timeout`.
async fn connect(opts: &ConnectOpts) -> anyhow::Result<AnyConnection> {
    retry_connect_errors(opts, AnyConnection::connect).await
}

/// Attempt an operation that may return errors like `ConnectionRefused`,
/// retrying up until `ops.connect_timeout`.
///
/// The closure is passed `&ops.database_url` for easy composition.
async fn retry_connect_errors<'a, F, Fut, T>(opts: &'a ConnectOpts, connect: F) -> anyhow::Result<T>
where
    F: FnMut(&'a str) -> Fut,
    Fut: Future<Output = sqlx::Result<T>> + 'a,
{
    sqlx::any::install_default_drivers();

    let db_url = opts.required_db_url()?;

    let (_, v) = {
        move |(mut ctx, db_url): (F, &'a str)| async move {
            let res = ctx(db_url).await;
            ((ctx, db_url), res)
        }
    }
    .retry(ExponentialBuilder::default().with_max_delay(Duration::from_secs(opts.connect_timeout)))
    .context((connect, db_url))
    .when(|err| {
        let sqlx::Error::Io(ref ioe) = err else {
            return false;
        };
        matches!(
            ioe.kind(),
            io::ErrorKind::ConnectionRefused
                | io::ErrorKind::ConnectionReset
                | io::ErrorKind::ConnectionAborted
        )
    })
    .await;

    Ok(v?)
}
