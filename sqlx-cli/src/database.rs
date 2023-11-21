use crate::migrate;
use crate::opt::ConnectOpts;
use console::style;
use promptly::{prompt, ReadlineError};
use sqlx::any::Any;
use sqlx::migrate::MigrateDatabase;

pub async fn create(connect_opts: &ConnectOpts) -> anyhow::Result<()> {
    // NOTE: only retry the idempotent action.
    // We're assuming that if this succeeds, then any following operations should also succeed.
    let exists = crate::retry_connect_errors(connect_opts, Any::database_exists).await?;

    if !exists {
        #[cfg(feature = "sqlite")]
        sqlx::sqlite::CREATE_DB_WAL.store(
            connect_opts.sqlite_create_db_wal,
            std::sync::atomic::Ordering::Release,
        );

        Any::create_database(connect_opts.required_db_url()?).await?;
    }

    Ok(())
}

pub async fn drop(connect_opts: &ConnectOpts, confirm: bool, force: bool) -> anyhow::Result<()> {
    if confirm && !ask_to_continue_drop(connect_opts.required_db_url()?) {
        return Ok(());
    }

    // NOTE: only retry the idempotent action.
    // We're assuming that if this succeeds, then any following operations should also succeed.
    let exists = crate::retry_connect_errors(connect_opts, Any::database_exists).await?;

    if exists {
        if force {
            Any::force_drop_database(connect_opts.required_db_url()?).await?;
        } else {
            Any::drop_database(connect_opts.required_db_url()?).await?;
        }
    }

    Ok(())
}

pub async fn reset(
    migration_source: &str,
    connect_opts: &ConnectOpts,
    confirm: bool,
    force: bool,
) -> anyhow::Result<()> {
    drop(connect_opts, confirm, force).await?;
    setup(migration_source, connect_opts).await
}

pub async fn setup(migration_source: &str, connect_opts: &ConnectOpts) -> anyhow::Result<()> {
    create(connect_opts).await?;
    migrate::run(migration_source, connect_opts, false, false, None).await
}

fn ask_to_continue_drop(db_url: &str) -> bool {
    loop {
        let r: Result<String, ReadlineError> =
            prompt(format!("Drop database at {}? (y/n)", style(db_url).cyan()));
        match r {
            Ok(response) => {
                if response == "n" || response == "N" {
                    return false;
                } else if response == "y" || response == "Y" {
                    return true;
                } else {
                    println!(
                        "Response not recognized: {}\nPlease type 'y' or 'n' and press enter.",
                        response
                    );
                }
            }
            Err(e) => {
                println!("{e}");
                return false;
            }
        }
    }
}
