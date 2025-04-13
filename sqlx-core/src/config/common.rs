/// Configuration shared by multiple components.
#[derive(Debug, Default)]
#[cfg_attr(
    feature = "sqlx-toml",
    derive(serde::Deserialize),
    serde(default, rename_all = "kebab-case", deny_unknown_fields)
)]
pub struct Config {
    /// Override the database URL environment variable.
    ///
    /// This is used by both the macros and `sqlx-cli`.
    ///
    /// Case-sensitive. Defaults to `DATABASE_URL`.
    ///
    /// Example: Multi-Database Project
    /// -------
    /// You can use multiple databases in the same project by breaking it up into multiple crates,
    /// then using a different environment variable for each.
    ///
    /// For example, with two crates in the workspace named `foo` and `bar`:
    ///
    /// #### `foo/sqlx.toml`
    /// ```toml
    /// [common]
    /// database-url-var = "FOO_DATABASE_URL"
    /// ```
    ///
    /// #### `bar/sqlx.toml`
    /// ```toml
    /// [common]
    /// database-url-var = "BAR_DATABASE_URL"
    /// ```
    ///
    /// #### `.env`
    /// ```text
    /// FOO_DATABASE_URL=postgres://postgres@localhost:5432/foo
    /// BAR_DATABASE_URL=postgres://postgres@localhost:5432/bar
    /// ```
    ///
    /// The query macros used in `foo` will use `FOO_DATABASE_URL`,
    /// and the ones used in `bar` will use `BAR_DATABASE_URL`.
    pub database_url_var: Option<String>,

    /// Settings for specific database drivers.
    ///
    /// These settings apply when checking queries, or when applying
    /// migrations via `sqlx-cli`. These settings *do not* apply when
    /// applying migrations via the macro, as that uses the run-time
    /// database connection configured by the application.
    pub drivers: Drivers,
}

impl Config {
    pub fn database_url_var(&self) -> &str {
        self.database_url_var.as_deref().unwrap_or("DATABASE_URL")
    }
}

/// Configuration for specific database drivers.
#[derive(Debug, Default)]
#[cfg_attr(
    feature = "sqlx-toml",
    derive(serde::Deserialize),
    serde(default, rename_all = "kebab-case")
)]
pub struct Drivers {
    /// Specify options for the SQLite driver.
    pub sqlite: SQLite,
}

/// Configuration for the SQLite database driver.
#[derive(Debug, Default)]
#[cfg_attr(
    feature = "sqlx-toml",
    derive(serde::Deserialize),
    serde(default, rename_all = "kebab-case")
)]
pub struct SQLite {
    /// Specify extensions to load.
    ///
    /// ### Example: Load the "uuid" and "vsv" extensions
    /// `sqlx.toml`:
    /// ```toml
    /// [common.drivers.sqlite]
    /// load-extensions = ["uuid", "vsv"]
    /// ```
    pub load_extensions: Vec<String>,
}
