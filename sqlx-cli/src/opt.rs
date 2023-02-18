use std::ops::{Deref, Not};

use clap::{Args, Parser};

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
    /// Saves metadata for all invocations of `query!` and related macros to a `.sqlx` directory
    /// in the current directory (or workspace root with `--workspace`), overwriting if needed.
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

        /// Generate a single workspace-level `.sqlx` folder.
        ///
        /// This option is intended for workspaces where multiple crates use SQLx. If there is only
        /// one, it is better to run `cargo sqlx prepare` without this option inside that crate.
        #[clap(long)]
        workspace: bool,

        /// Arguments to be passed to `cargo rustc ...`.
        #[clap(last = true)]
        args: Vec<String>,

        #[clap(flatten)]
        connect_opts: ConnectOpts,
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
        #[clap(flatten)]
        connect_opts: ConnectOpts,
    },

    /// Drops the database specified in your DATABASE_URL.
    Drop {
        #[clap(flatten)]
        confirmation: Confirmation,

        #[clap(flatten)]
        connect_opts: ConnectOpts,
    },

    /// Drops the database specified in your DATABASE_URL, re-creates it, and runs any pending migrations.
    Reset {
        #[clap(flatten)]
        confirmation: Confirmation,

        #[clap(flatten)]
        source: Source,

        #[clap(flatten)]
        connect_opts: ConnectOpts,
    },

    /// Creates the database specified in your DATABASE_URL and runs any pending migrations.
    Setup {
        #[clap(flatten)]
        source: Source,

        #[clap(flatten)]
        connect_opts: ConnectOpts,
    },
}

/// Group of commands for creating and running migrations.
#[derive(Parser, Debug)]
pub struct MigrateOpt {
    #[clap(subcommand)]
    pub command: MigrateCommand,
}

#[derive(Parser, Debug)]
pub enum MigrateCommand {
    /// Create a new migration with the given description,
    /// and the current time as the version.
    Add {
        description: String,

        #[clap(flatten)]
        source: Source,

        /// If true, creates a pair of up and down migration files with same version
        /// else creates a single sql file
        #[clap(short)]
        reversible: bool,
    },

    /// Run all pending migrations.
    Run {
        #[clap(flatten)]
        source: Source,

        /// List all the migrations to be run without applying
        #[clap(long)]
        dry_run: bool,

        #[clap(flatten)]
        ignore_missing: IgnoreMissing,

        #[clap(flatten)]
        connect_opts: ConnectOpts,
    },

    /// Revert the latest migration with a down file.
    Revert {
        #[clap(flatten)]
        source: Source,

        /// List the migration to be reverted without applying
        #[clap(long)]
        dry_run: bool,

        #[clap(flatten)]
        ignore_missing: IgnoreMissing,

        #[clap(flatten)]
        connect_opts: ConnectOpts,
    },

    /// List all available migrations.
    Info {
        #[clap(flatten)]
        source: Source,

        #[clap(flatten)]
        connect_opts: ConnectOpts,
    },

    /// Generate a `build.rs` to trigger recompilation when a new migration is added.
    ///
    /// Must be run in a Cargo project root.
    BuildScript {
        #[clap(flatten)]
        source: Source,

        /// Overwrite the build script if it already exists.
        #[clap(long)]
        force: bool,
    },
}

/// Argument for the migration scripts source.
#[derive(Args, Debug)]
pub struct Source {
    /// Path to folder containing migrations.
    #[clap(long, default_value = "migrations")]
    source: String,
}

impl Deref for Source {
    type Target = String;

    fn deref(&self) -> &Self::Target {
        &self.source
    }
}

/// Argument for the database URL.
#[derive(Args, Debug)]
pub struct ConnectOpts {
    /// Location of the DB, by default will be read from the DATABASE_URL env var
    #[clap(long, short = 'D', env)]
    pub database_url: String,

    /// The maximum time, in seconds, to try connecting to the database server before
    /// returning an error.
    #[clap(long, default_value = "10")]
    pub connect_timeout: u64,

    /// Set whether or not to create SQLite databases in Write-Ahead Log (WAL) mode:
    /// https://www.sqlite.org/wal.html
    ///
    /// WAL mode is enabled by default for SQLite databases created by `sqlx-cli`.
    ///
    /// However, if your application sets a `journal_mode` on `SqliteConnectOptions` to something
    /// other than `Wal`, then it will have to take the database file out of WAL mode on connecting,
    /// which requires an exclusive lock and may return a `database is locked` (`SQLITE_BUSY`) error.
    #[cfg(feature = "sqlite")]
    #[clap(long, action = clap::ArgAction::Set, default_value = "true")]
    pub sqlite_create_db_wal: bool,
}

/// Argument for automatic confirmation.
#[derive(Args, Copy, Clone, Debug)]
pub struct Confirmation {
    /// Automatic confirmation. Without this option, you will be prompted before dropping
    /// your database.
    #[clap(short)]
    pub yes: bool,
}

/// Argument for ignoring applied migrations that were not resolved.
#[derive(Args, Copy, Clone, Debug)]
pub struct IgnoreMissing {
    /// Ignore applied migrations that are missing in the resolved migrations
    #[clap(long)]
    ignore_missing: bool,
}

impl Deref for IgnoreMissing {
    type Target = bool;

    fn deref(&self) -> &Self::Target {
        &self.ignore_missing
    }
}

impl Not for IgnoreMissing {
    type Output = bool;

    fn not(self) -> Self::Output {
        !self.ignore_missing
    }
}
