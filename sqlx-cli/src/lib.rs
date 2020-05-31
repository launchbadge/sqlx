use dotenv::dotenv;

use structopt::StructOpt;

mod migrator;

mod db;
mod migration;
mod prepare;

#[derive(StructOpt, Debug)]
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
    /// Outputs to `sqlx-data.json` in the current directory, overwriting it if it already exists.
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

        /// Any arguments to pass to `cargo rustc`;
        /// Cargo args (preceding `--` in `cargo rustc ... -- ...`) only.
        #[structopt(name = "Cargo args", last = true)]
        cargo_args: Vec<String>,
    },
}

/// Generate and run migrations
#[derive(StructOpt, Debug)]
pub enum MigrationCommand {
    /// Create a new migration with the given name,
    /// using the current time as the version
    Add { name: String },

    /// Run all pending migrations
    Run,

    /// List all migrations
    List,
}

/// Create or drops database depending on your connection string
#[derive(StructOpt, Debug)]
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

        Command::Prepare {
            check: false,
            cargo_args,
        } => prepare::run(cargo_args)?,

        Command::Prepare {
            check: true,
            cargo_args,
        } => prepare::check(cargo_args)?,
    };

    Ok(())
}
