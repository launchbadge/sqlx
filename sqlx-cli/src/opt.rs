use clap::Clap;

#[derive(Clap, Debug)]
pub struct Opt {
    #[clap(subcommand)]
    pub command: Command,

    #[clap(short = 'D', long)]
    pub database_url: Option<String>,
}

#[derive(Clap, Debug)]
pub enum Command {
    #[clap(alias = "db")]
    Database(DatabaseOpt),

    /// Generate query metadata to support offline compile-time verification.
    ///
    /// Saves metadata for all invocations of `query!` and related macros to `sqlx-data.json`
    /// in the current directory, overwriting if needed.
    ///
    /// During project compilation, the absence of the `DATABASE_URL` environment variable or
    /// the presence of `SQLX_OFFLINE` will constrain the compile-time verification to only
    /// read from the cached query metadata.
    #[clap(alias = "prep")]
    Prepare {
        /// Run in 'check' mode. Exits with 0 if the query metadata is up-to-date. Exits with
        /// 1 if the query metadata needs updating.
        #[clap(long)]
        check: bool,

        /// Arguments to be passed to `cargo rustc ...`.
        #[clap(last = true)]
        args: Vec<String>,
    },

    #[clap(alias = "mig")]
    Migrate(MigrateOpt),
}

/// Group of commands for creating and dropping your database.
#[derive(Clap, Debug)]
pub struct DatabaseOpt {
    #[clap(subcommand)]
    pub command: DatabaseCommand,
}

#[derive(Clap, Debug)]
pub enum DatabaseCommand {
    /// Creates the database specified in your DATABASE_URL.
    Create,

    /// Drops the database specified in your DATABASE_URL.
    Drop {
        /// Automatic confirmation. Without this option, you will be prompted before dropping
        /// your database.
        #[clap(short)]
        yes: bool,
    },

    /// Drops the database specified in your DATABASE_URL, re-creates it, and runs any pending migrations.
    Reset {
        /// Automatic confirmation. Without this option, you will be prompted before dropping
        /// your database.
        #[clap(short)]
        yes: bool,

        /// Path to folder containing migrations. Defaults to 'migrations'
        #[clap(long, default_value = "migrations")]
        source: String,
    },

    /// Creates the database specified in your DATABASE_URL and runs any pending migrations.
    Setup {
        /// Path to folder containing migrations. Defaults to 'migrations'
        #[clap(long, default_value = "migrations")]
        source: String,
    },
}

/// Group of commands for creating and running migrations.
#[derive(Clap, Debug)]
pub struct MigrateOpt {
    /// Path to folder containing migrations. Defaults to 'migrations'
    #[clap(long, default_value = "migrations")]
    pub source: String,

    #[clap(subcommand)]
    pub command: MigrateCommand,
}

#[derive(Clap, Debug)]
pub enum MigrateCommand {
    /// Create a new migration with the given description,
    /// and the current time as the version.
    Add { description: String },

    /// Run all pending migrations.
    Run,

    /// List all available migrations.
    Info,
}
