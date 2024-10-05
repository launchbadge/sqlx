use std::io;
use std::path::{PathBuf};
use std::time::Duration;

use anyhow::{Context, Result};
use futures::{Future, TryFutureExt};

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

pub use sqlx::_unstable::config::{self, Config};

pub async fn run(opt: Opt) -> Result<()> {
    let config = config_from_current_dir().await?;

    match opt.command {
        Command::Migrate(migrate) => match migrate.command {
            MigrateCommand::Add(opts)=> migrate::add(config, opts).await?,
            MigrateCommand::Run {
                source,
                dry_run,
                ignore_missing,
                mut connect_opts,
                target_version,
            } => {
                connect_opts.populate_db_url(config)?;

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
                mut connect_opts,
                target_version,
            } => {
                connect_opts.populate_db_url(config)?;

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
                mut connect_opts,
            } => {
                connect_opts.populate_db_url(config)?;

                migrate::info(&source, &connect_opts).await?
            },
            MigrateCommand::BuildScript { source, force } => migrate::build_script(&source, force)?,
        },

        Command::Database(database) => match database.command {
            DatabaseCommand::Create { mut connect_opts } => {
                connect_opts.populate_db_url(config)?;
                database::create(&connect_opts).await?
            },
            DatabaseCommand::Drop {
                confirmation,
                mut connect_opts,
                force,
            } => {
                connect_opts.populate_db_url(config)?;
                database::drop(&connect_opts, !confirmation.yes, force).await?
            },
            DatabaseCommand::Reset {
                confirmation,
                source,
                mut connect_opts,
                force,
            } => {
                connect_opts.populate_db_url(config)?;
                database::reset(&source, &connect_opts, !confirmation.yes, force).await?
            },
            DatabaseCommand::Setup {
                source,
                mut connect_opts,
            } => {
                connect_opts.populate_db_url(config)?;
                database::setup(&source, &connect_opts).await?
            },
        },

        Command::Prepare {
            check,
            all,
            workspace,
            mut connect_opts,
            args,
        } => {
            connect_opts.populate_db_url(config)?;
            prepare::run(check, all, workspace, connect_opts, args).await?
        },

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
async fn retry_connect_errors<'a, F, Fut, T>(
    opts: &'a ConnectOpts,
    mut connect: F,
) -> anyhow::Result<T>
where
    F: FnMut(&'a str) -> Fut,
    Fut: Future<Output = sqlx::Result<T>> + 'a,
{
    sqlx::any::install_default_drivers();

    let db_url = opts.expect_db_url()?;

    backoff::future::retry(
        backoff::ExponentialBackoffBuilder::new()
            .with_max_elapsed_time(Some(Duration::from_secs(opts.connect_timeout)))
            .build(),
        || {
            connect(db_url).map_err(|e| -> backoff::Error<anyhow::Error> {
                if let sqlx::Error::Io(ref ioe) = e {
                    match ioe.kind() {
                        io::ErrorKind::ConnectionRefused
                        | io::ErrorKind::ConnectionReset
                        | io::ErrorKind::ConnectionAborted => {
                            return backoff::Error::transient(e.into());
                        }
                        _ => (),
                    }
                }

                backoff::Error::permanent(e.into())
            })
        },
    )
    .await
}

async fn config_from_current_dir() -> anyhow::Result<&'static Config> {
    // Tokio does file I/O on a background task anyway
    tokio::task::spawn_blocking(|| {
        let path = PathBuf::from("sqlx.toml");

        if path.exists() {
            eprintln!("Found `sqlx.toml` in current directory; reading...");
        }

        Config::read_with_or_default(move || Ok(path))
    })
        .await
        .context("unexpected error loading config")
}
