use std::path::{Path, PathBuf};
use std::str::FromStr;
use url::Url;

use crate::error::BoxDynError;

// TODO: Look at go-sqlite for option ideas
// TODO: journal_mode

/// Options and flags which can be used to configure a SQLite connection.
pub struct SqliteConnectOptions {
    pub(crate) filename: PathBuf,
    pub(crate) in_memory: bool,
}

impl SqliteConnectOptions {
    pub fn new() -> Self {
        Self {
            filename: PathBuf::from(":memory:"),
            in_memory: false,
        }
    }
}

impl FromStr for SqliteConnectOptions {
    type Err = BoxDynError;

    fn from_str(mut s: &str) -> Result<Self, Self::Err> {
        let mut options = Self {
            filename: PathBuf::new(),
            in_memory: false,
        };

        // remove scheme
        s = s
            .trim_start_matches("sqlite://")
            .trim_start_matches("sqlite:");

        if s == ":memory:" {
            options.in_memory = true;
        } else {
            options.filename = s.parse()?;
        }

        Ok(options)
    }
}
