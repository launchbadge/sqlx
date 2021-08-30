use crate::migrate;
use console::style;
use promptly::{prompt, ReadlineError};
use sqlx::any::Any;
use sqlx::migrate::MigrateDatabase;

pub async fn create(uri: &str) -> anyhow::Result<()> {
    if !Any::database_exists(uri).await? {
        Any::create_database(uri).await?;
    }

    Ok(())
}

pub async fn drop(uri: &str, confirm: bool) -> anyhow::Result<()> {
    if confirm && !ask_to_continue(uri) {
        return Ok(());
    }

    if Any::database_exists(uri).await? {
        Any::drop_database(uri).await?;
    }

    Ok(())
}

pub async fn reset(migration_source: &str, uri: &str, confirm: bool) -> anyhow::Result<()> {
    drop(uri, confirm).await?;
    setup(migration_source, uri).await
}

pub async fn setup(migration_source: &str, uri: &str) -> anyhow::Result<()> {
    create(uri).await?;
    migrate::run(migration_source, uri, false, false).await
}

fn ask_to_continue(uri: &str) -> bool {
    loop {
        let r: Result<String, ReadlineError> =
            prompt(format!("Drop database at {}? (y/n)", style(uri).cyan()));
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
                println!("{}", e);
                return false;
            }
        }
    }
}
