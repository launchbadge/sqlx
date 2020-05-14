





use dotenv::dotenv;

use structopt::StructOpt;





mod migrator;

mod db;
mod migration;
mod prepare;



/// Sqlx commandline tool
#[derive(StructOpt, Debug)]
#[structopt(name = "Sqlx")]
pub enum Command {
    #[structopt(alias = "mig")]
    Migrate(MigrationCommand),

    #[structopt(alias = "db")]
    Database(DatabaseCommand),

    /// Enables offline mode for a project utilizing `query!()` and related macros.
    /// May only be run as `cargo sqlx prepare`.
    ///
    /// Saves data for all invocations of `query!()` and friends in the project so that it may be
    /// built in offline mode, i.e. so compilation does not require connecting to a running database.
    /// Outputs to `sqlx-data.json` in the current directory.
    ///
    /// Offline mode can be activated simply by removing `DATABASE_URL` from the environment or
    /// building without a `.env` file.
    #[structopt(alias = "prep")]
    Prepare {
        /// If this flag is passed, instead of overwriting `sqlx-data.json` in the current directory,
        /// that file is loaded and compared against the current output of the prepare step; if
        /// there is a mismatch, an error is reported and the process exits with a nonzero exit code.
        ///
        /// Intended for use in CI.
        #[structopt(long)]
        check: bool,
    },
}

/// Adds and runs migrations. Alias: mig
#[derive(StructOpt, Debug)]
#[structopt(name = "Sqlx migrator")]
pub enum MigrationCommand {
    /// Add new migration with name <timestamp>_<migration_name>.sql
    Add { name: String },

    /// Run all migrations
    Run,

    /// List all migrations
    List,
}

/// Create or drops database depending on your connection string. Alias: db
#[derive(StructOpt, Debug)]
#[structopt(name = "Sqlx migrator")]
pub enum DatabaseCommand {
    /// Create database in url
    Create,

    /// Drop database in url
    Drop,
}

pub async fn run(cmd: Command) -> anyhow::Result<()> {
    dotenv().ok();

    match cmd {
        Command::Migrate(migrate) => match migrate {
            MigrationCommand::Add { name } => migration::add_file(&name)?,
            MigrationCommand::Run => migration::run().await?,
            MigrationCommand::List => migration::list().await?,
        },
        Command::Database(database) => match database {
            DatabaseCommand::Create => db::run_create().await?,
            DatabaseCommand::Drop => db::run_drop().await?,
        },
        Command::Prepare { check: false } => prepare::run()?,
        Command::Prepare { check: true } => prepare::check()?,
    };

    Ok(())
}
