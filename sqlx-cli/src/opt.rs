use clap::Parser;

#[derive(Parser, Debug)]
#[clap(version, about, author)]
pub struct Opt {
    #[clap(subcommand)]
    pub command: Command,
}

#[derive(Parser, Debug)]
pub enum Command {
    #[clap(alias = "db")]
    Database(DatabaseOpt),

    /// Generate query metadata to support offline compile-time verification.
    ///
    /// Saves metadata for all invocations of `query!` and related macros to `sqlx-data.json`
    /// in the current directory, overwriting if needed.
    ///
    /// During project compilation, the absence of the `DATABASE_URL` environment variable or
    /// the presence of `SQLX_OFFLINE` (with a value of `true` or `1`) will constrain the
    /// compile-time verification to only read from the cached query metadata.
    #[clap(alias = "prep")]
    Prepare {
        /// Run in 'check' mode. Exits with 0 if the query metadata is up-to-date. Exits with
        /// 1 if the query metadata needs updating.
        #[clap(long)]
        check: bool,

        /// Generate a single top-level `sqlx-data.json` file when using a cargo workspace.
        #[clap(long)]
        merged: bool,

        /// Arguments to be passed to `cargo rustc ...`.
        #[clap(last = true)]
        args: Vec<String>,

        /// Location of the DB, by default will be read from the DATABASE_URL env var
        #[clap(long, short = 'D', env)]
        database_url: String,
    },

    #[clap(alias = "mig")]
    Migrate(MigrateOpt),
}

/// Group of commands for creating and dropping your database.
#[derive(Parser, Debug)]
pub struct DatabaseOpt {
    #[clap(subcommand)]
    pub command: DatabaseCommand,
}

#[derive(Parser, Debug)]
pub enum DatabaseCommand {
    /// Creates the database specified in your DATABASE_URL.
    Create {
        /// Location of the DB, by default will be read from the DATABASE_URL env var
        #[clap(long, short = 'D', env)]
        database_url: String,
    },

    /// Drops the database specified in your DATABASE_URL.
    Drop {
        /// Automatic confirmation. Without this option, you will be prompted before dropping
        /// your database.
        #[clap(short)]
        yes: bool,

        /// Location of the DB, by default will be read from the DATABASE_URL env var
        #[clap(long, short = 'D', env)]
        database_url: String,
    },

    /// Drops the database specified in your DATABASE_URL, re-creates it, and runs any pending migrations.
    Reset {
        /// Automatic confirmation. Without this option, you will be prompted before dropping
        /// your database.
        #[clap(short)]
        yes: bool,

        /// Path to folder containing migrations.
        #[clap(long, default_value = "migrations")]
        source: String,

        /// Location of the DB, by default will be read from the DATABASE_URL env var
        #[clap(long, short = 'D', env)]
        database_url: String,
    },

    /// Creates the database specified in your DATABASE_URL and runs any pending migrations.
    Setup {
        /// Path to folder containing migrations.
        #[clap(long, default_value = "migrations")]
        source: String,

        /// Location of the DB, by default will be read from the DATABASE_URL env var
        #[clap(long, short = 'D', env)]
        database_url: String,
    },
}

/// Group of commands for creating and running migrations.
#[derive(Parser, Debug)]
pub struct MigrateOpt {
    /// Path to folder containing migrations.
    #[clap(long, default_value = "migrations")]
    pub source: String,

    #[clap(subcommand)]
    pub command: MigrateCommand,
}

#[derive(Parser, Debug)]
pub enum MigrateCommand {
    /// Create a new migration with the given description,
    /// and the current time as the version.
    Add {
        description: String,

        /// If true, creates a pair of up and down migration files with same version
        /// else creates a single sql file
        #[clap(short)]
        reversible: bool,
    },

    /// Run all pending migrations.
    Run {
        /// List all the migrations to be run without applying
        #[clap(long)]
        dry_run: bool,

        /// Ignore applied migrations that missing in the resolved migrations
        #[clap(long)]
        ignore_missing: bool,

        /// Location of the DB, by default will be read from the DATABASE_URL env var
        #[clap(long, short = 'D', env)]
        database_url: String,
    },

    /// Revert the latest migration with a down file.
    Revert {
        /// List the migration to be reverted without applying
        #[clap(long)]
        dry_run: bool,

        /// Ignore applied migrations that missing in the resolved migrations
        #[clap(long)]
        ignore_missing: bool,

        /// Location of the DB, by default will be read from the DATABASE_URL env var
        #[clap(long, short = 'D', env)]
        database_url: String,
    },

    /// List all available migrations.
    Info {
        /// Location of the DB, by default will be read from the DATABASE_URL env var
        #[clap(long, env)]
        database_url: String,
    },

    /// Generate a `build.rs` to trigger recompilation when a new migration is added.
    ///
    /// Must be run in a Cargo project root.
    BuildScript {
        /// Overwrite the build script if it already exists.
        #[clap(long)]
        force: bool,
    },
}
