use crate::migrator::DatabaseMigrator;
use dialoguer::Confirmation;

use anyhow::bail;

pub async fn run_create() -> anyhow::Result<()> {
    let migrator = crate::migrator::get()?;

    if !migrator.can_create_database() {
        bail!(
            "Database creation is not implemented for {}",
            migrator.database_type()
        );
    }

    let db_name = migrator.get_database_name()?;
    let db_exists = migrator.check_if_database_exists(&db_name).await?;

    if !db_exists {
        println!("Creating database: {}", db_name);
        Ok(migrator.create_database(&db_name).await?)
    } else {
        println!("Database already exists, aborting");
        Ok(())
    }
}

pub async fn run_drop() -> anyhow::Result<()> {
    let migrator = crate::migrator::get()?;

    if !migrator.can_drop_database() {
        bail!(
            "Database drop is not implemented for {}",
            migrator.database_type()
        );
    }

    let db_name = migrator.get_database_name()?;
    let db_exists = migrator.check_if_database_exists(&db_name).await?;

    if db_exists {
        if !Confirmation::new()
            .with_text("\nAre you sure you want to drop the database: {}?")
            .default(false)
            .interact()?
        {
            println!("Aborting");
            return Ok(());
        }

        println!("Dropping database: {}", db_name);
        Ok(migrator.drop_database(&db_name).await?)
    } else {
        println!("Database does not exists, aborting");
        Ok(())
    }
}
