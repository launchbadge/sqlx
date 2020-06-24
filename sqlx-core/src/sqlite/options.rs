use std::path::PathBuf;
use std::{io, str::FromStr};

use crate::error::BoxDynError;

// TODO: Look at go-sqlite for option ideas
// TODO: journal_mode

/// Options and flags which can be used to configure a SQLite connection.
pub struct SqliteConnectOptions {
    pub(crate) filename: PathBuf,
    pub(crate) in_memory: bool,
    pub(crate) statement_cache_size: usize,
}

impl Default for SqliteConnectOptions {
    fn default() -> Self {
        Self::new()
    }
}

impl SqliteConnectOptions {
    pub fn new() -> Self {
        Self {
            filename: PathBuf::from(":memory:"),
            in_memory: false,
            statement_cache_size: 100,
        }
    }
}

impl FromStr for SqliteConnectOptions {
    type Err = BoxDynError;

    fn from_str(mut s: &str) -> Result<Self, Self::Err> {
        let mut options = Self {
            filename: PathBuf::new(),
            in_memory: false,
            statement_cache_size: 100,
        };

        // remove scheme
        s = s
            .trim_start_matches("sqlite://")
            .trim_start_matches("sqlite:");

        let mut splitted = s.split("?");

        match splitted.next() {
            Some(":memory:") => options.in_memory = true,
            Some(s) => options.filename = s.parse()?,
            None => unreachable!(),
        }

        match splitted.next().map(|s| s.split("=")) {
            Some(mut splitted) => {
                if splitted.next() == Some("statement-cache-size") {
                    options.statement_cache_size = splitted
                        .next()
                        .ok_or_else(|| {
                            io::Error::new(io::ErrorKind::InvalidInput, "Invalid connection string")
                        })?
                        .parse()?
                }
            }
            _ => (),
        }

        Ok(options)
    }
}
