use std::collections::BTreeSet;

/// Configuration for migrations when executed using `sqlx::migrate!()` or through `sqlx-cli`.
///
/// ### Note
/// A manually constructed [`Migrator`][crate::migrate::Migrator] will not be aware of these
/// configuration options. We recommend using `sqlx::migrate!()` instead.
///
/// ### Warning: Potential Data Loss or Corruption!
/// Many of these options, if changed after migrations are set up,
/// can result in data loss or corruption of a production database
/// if the proper precautions are not taken.
///
/// Be sure you know what you are doing and that you read all relevant documentation _thoroughly_.
#[derive(Debug, Default, serde::Deserialize)]
#[serde(default)]
pub struct Config {
    /// Override the name of the table used to track executed migrations.
    ///
    /// May be schema-qualified and/or contain quotes. Defaults to `_sqlx_migrations`.
    ///
    /// Potentially useful for multi-tenant databases.
    ///
    /// ### Warning: Potential Data Loss or Corruption!
    /// Changing this option for a production database will likely result in data loss or corruption
    /// as the migration machinery will no longer be aware of what migrations have been applied
    /// and will attempt to re-run them.
    ///
    /// You should create the new table as a copy of the existing migrations table (with contents!),
    /// and be sure all instances of your application have been migrated to the new
    /// table before deleting the old one.
    ///
    /// ### Example
    /// `sqlx.toml`:
    /// ```toml
    /// [migrate]
    /// # Put `_sqlx_migrations` in schema `foo`
    /// table_name = "foo._sqlx_migrations"
    /// ```
    pub table_name: Option<Box<str>>,

    /// Override the directory used for migrations files.
    ///
    /// Relative to the crate root for `sqlx::migrate!()`, or the current directory for `sqlx-cli`.
    pub migrations_dir: Option<Box<str>>,

    /// Specify characters that should be ignored when hashing migrations.
    ///
    /// Any characters contained in the given array will be dropped when a migration is hashed.
    ///
    /// ### Warning: May Change Hashes for Existing Migrations
    /// Changing the characters considered in hashing migrations will likely
    /// change the output of the hash.
    ///
    /// This may require manual rectification for deployed databases.
    ///
    /// ### Example: Ignore Carriage Return (`<CR>` | `\r`)
    /// Line ending differences between platforms can result in migrations having non-repeatable
    /// hashes. The most common culprit is the carriage return (`<CR>` | `\r`), which Windows
    /// uses in its line endings alongside line feed (`<LF>` | `\n`), often written `CRLF` or `\r\n`,
    /// whereas Linux and macOS use only line feeds.
    ///
    /// `sqlx.toml`:
    /// ```toml
    /// [migrate]
    /// ignored_chars = ["\r"]
    /// ```
    ///
    /// For projects using Git, this can also be addressed using [`.gitattributes`]:
    ///
    /// ```text
    /// # Force newlines in migrations to be line feeds on all platforms
    /// migrations/*.sql text eol=lf
    /// ```
    ///
    /// This may require resetting or re-checking out the migrations files to take effect.
    ///
    /// [`.gitattributes`]: https://git-scm.com/docs/gitattributes
    ///
    /// ### Example: Ignore all Whitespace Characters
    /// To make your migrations amenable to reformatting, you may wish to tell SQLx to ignore
    /// _all_ whitespace characters in migrations.
    ///
    /// ##### Warning: Beware Syntatically Significant Whitespace!
    /// If your migrations use string literals or quoted identifiers which contain whitespace,
    /// this configuration will cause the migration machinery to ignore some changes to these.
    /// This may result in a mismatch between the development and production versions of
    /// your database.
    ///
    /// `sqlx.toml`:
    /// ```toml
    /// [migrate]
    /// # Ignore common whitespace characters when hashing
    /// ignored_chars = [" ", "\t", "\r", "\n"]  # Space, tab, CR, LF
    /// ```
    // Likely lower overhead for small sets than `HashSet`.
    pub ignored_chars: BTreeSet<char>,

    /// Specify the default type of migration that `sqlx migrate create` should create by default.
    ///
    /// ### Example: Use Reversible Migrations by Default
    /// `sqlx.toml`:
    /// ```toml
    /// [migrate]
    /// default_type = "reversible"
    /// ```
    pub default_type: DefaultMigrationType,

    /// Specify the default scheme that `sqlx migrate create` should use for version integers.
    ///
    /// ### Example: Use Sequential Versioning by Default
    /// `sqlx.toml`:
    /// ```toml
    /// [migrate]
    /// default_versioning = "sequential"
    /// ```
    pub default_versioning: DefaultVersioning,
}

/// The default type of migration that `sqlx migrate create` should create by default.
#[derive(Debug, Default, PartialEq, Eq, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DefaultMigrationType {
    /// Create the same migration type as that of the latest existing migration,
    /// or `Simple` otherwise.
    #[default]
    Inferred,

    /// Create a non-reversible migration (`<VERSION>_<DESCRIPTION>.sql`).
    Simple,

    /// Create a reversible migration (`<VERSION>_<DESCRIPTION>.up.sql` and `[...].down.sql`).
    Reversible,
}

/// The default scheme that `sqlx migrate create` should use for version integers.
#[derive(Debug, Default, PartialEq, Eq, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DefaultVersioning {
    /// Infer the versioning scheme from existing migrations:
    ///
    /// * If the versions of the last two migrations differ by `1`, infer `Sequential`.
    /// * If only one migration exists and has version `1`, infer `Sequential`.
    /// * Otherwise, infer `Timestamp`.
    #[default]
    Inferred,

    /// Use UTC timestamps for migration versions.
    ///
    /// This is the recommended versioning format as it's less likely to collide when multiple
    /// developers are creating migrations on different branches.
    ///
    /// The exact timestamp format is unspecified.
    Timestamp,

    /// Use sequential integers for migration versions.
    Sequential,
}
