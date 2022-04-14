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

        #[clap(flatten)]
        database_url: DatabaseUrl,
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
        database_url: DatabaseUrl,
    },

    /// Drops the database specified in your DATABASE_URL.
    Drop {
        #[clap(flatten)]
        confirmation: Confirmation,

        #[clap(flatten)]
        database_url: DatabaseUrl,
    },

    /// Drops the database specified in your DATABASE_URL, re-creates it, and runs any pending migrations.
    Reset {
        #[clap(flatten)]
        confirmation: Confirmation,

        #[clap(flatten)]
        source: Source,

        #[clap(flatten)]
        database_url: DatabaseUrl,
    },

    /// Creates the database specified in your DATABASE_URL and runs any pending migrations.
    Setup {
        #[clap(flatten)]
        source: Source,

        #[clap(flatten)]
        database_url: DatabaseUrl,
    },
}

/// Group of commands for creating and running migrations.
#[derive(Parser, Debug)]
pub struct MigrateOpt {
    /// Path to folder containing migrations.
    /// Warning: deprecated, use <SUBCOMMAND> --source <SOURCE>
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

        #[clap(flatten)]
        source: SourceOverride,

        /// If true, creates a pair of up and down migration files with same version
        /// else creates a single sql file
        #[clap(short)]
        reversible: bool,
    },

    /// Run all pending migrations.
    Run {
        #[clap(flatten)]
        source: SourceOverride,

        /// List all the migrations to be run without applying
        #[clap(long)]
        dry_run: bool,

        #[clap(flatten)]
        ignore_missing: IgnoreMissing,

        #[clap(flatten)]
        database_url: DatabaseUrl,
    },

    /// Revert the latest migration with a down file.
    Revert {
        #[clap(flatten)]
        source: SourceOverride,

        /// List the migration to be reverted without applying
        #[clap(long)]
        dry_run: bool,

        #[clap(flatten)]
        ignore_missing: IgnoreMissing,

        #[clap(flatten)]
        database_url: DatabaseUrl,
    },

    /// List all available migrations.
    Info {
        #[clap(flatten)]
        source: SourceOverride,

        #[clap(flatten)]
        database_url: DatabaseUrl,
    },

    /// Generate a `build.rs` to trigger recompilation when a new migration is added.
    ///
    /// Must be run in a Cargo project root.
    BuildScript {
        #[clap(flatten)]
        source: SourceOverride,

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

/// Argument for overriding migration scripts source.
// Note: once `MigrateOpt.source` is removed, usage can be replaced with `Source`.
#[derive(Args, Debug)]
pub struct SourceOverride {
    /// Path to folder containing migrations [default: migrations]
    #[clap(long)]
    source: Option<String>,
}

impl SourceOverride {
    /// Override command's `source` flag value with subcommand's
    /// `source` flag value when provided.
    #[inline]
    pub(super) fn resolve<'a>(&'a self, source: &'a str) -> &'a str {
        match self.source {
            Some(ref source) => source,
            None => source,
        }
    }
}

/// Argument for the database URL.
#[derive(Args, Debug)]
pub struct DatabaseUrl {
    /// Location of the DB, by default will be read from the DATABASE_URL env var
    #[clap(long, short = 'D', env)]
    database_url: String,
}

impl Deref for DatabaseUrl {
    type Target = String;

    fn deref(&self) -> &Self::Target {
        &self.database_url
    }
}

/// Argument for automatic confirmantion.
#[derive(Args, Copy, Clone, Debug)]
pub struct Confirmation {
    /// Automatic confirmation. Without this option, you will be prompted before dropping
    /// your database.
    #[clap(short)]
    yes: bool,
}

impl Deref for Confirmation {
    type Target = bool;

    fn deref(&self) -> &Self::Target {
        &self.yes
    }
}

impl Not for Confirmation {
    type Output = bool;

    fn not(self) -> Self::Output {
        !self.yes
    }
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
