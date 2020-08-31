use crate::migrate;
use console::style;
use dialoguer::Confirm;
use sqlx::any::Any;
use sqlx::migrate::MigrateDatabase;

pub async fn create(uri: &str) -> anyhow::Result<()> {
    if !Any::database_exists(uri).await? {
        Any::create_database(uri).await?;
    }

    Ok(())
}

pub async fn drop(uri: &str, confirm: bool) -> anyhow::Result<()> {
    if confirm
        && !Confirm::new()
            .with_prompt(format!(
                "\nAre you sure you want to drop the database at {}?",
                style(uri).cyan()
            ))
            .default(false)
            .interact()?
    {
        return Ok(());
    }

    if Any::database_exists(uri).await? {
        Any::drop_database(uri).await?;
    }

    Ok(())
}

pub async fn reset(uri: &str, confirm: bool) -> anyhow::Result<()> {
    drop(uri, confirm).await?;
    setup(uri).await
}

pub async fn setup(uri: &str) -> anyhow::Result<()> {
    create(uri).await?;
    migrate::run(uri).await
}
