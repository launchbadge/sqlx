//! `sqlx.toml` config.

use std::collections::HashMap;
use std::path::Path;

#[derive(serde::Deserialize, Default)]
#[serde(rename_all = "kebab-case")]
#[serde(default)]
pub struct Config {
    /// Override the environment variable used to connect to the database (default `DATABASE_URL`).
    pub url_var: Option<String>,

    /// Configure SQL -> Rust type mappings.
    pub types: TypesConfig,

    // Possible future extensions:
    // * enable out-of-tree drivers
    // * inheritance
    // * create a shadowed set of macros with a different config:
    // https://github.com/launchbadge/sqlx/issues/121#issuecomment-609092100
}

#[derive(serde::Deserialize, Default)]
#[serde(rename_all = "kebab-case")]
#[serde(default)]
pub struct TypesConfig {
    /// Choose the preferred crate for time-related types (`TIMESTAMP`, `DATETIME`, `TIME`, `DATE`).
    pub time_crate: Option<TimeCrate>,

    /// Choose the preferred crate for `NUMERIC`.
    pub numeric_crate: Option<NumericCrate>,

    /// Global type overrides (SQL type name -> fully qualified Rust path).
    pub r#override: HashMap<String, String>
}

#[derive(serde::Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum TimeCrate {
    Chrono,
    Time
}

#[derive(serde::Deserialize)]
pub enum NumericCrate {
    #[serde(rename = "rust_decimal")]
    RustDecimal,
    #[serde(rename = "bigdecimal")]
    BigDecimal
}

pub fn load(path: &Path) -> crate::Result<Config> {

}
