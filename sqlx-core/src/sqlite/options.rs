use std::path::PathBuf;
use std::str::FromStr;

use crate::error::BoxDynError;

// TODO: Look at go-sqlite for option ideas
// TODO: journal_mode

/// Options and flags which can be used to configure a SQLite connection.
pub struct SqliteConnectOptions {
    pub(crate) filename: PathBuf,
    pub(crate) in_memory: bool,
    pub(crate) statement_cache_capacity: usize,
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
            statement_cache_capacity: 100,
        }
    }

    /// Sets the capacity of the connection's statement cache in a number of stored
    /// distinct statements. Caching is handled using LRU, meaning when the
    /// amount of queries hits the defined limit, the oldest statement will get
    /// dropped.
    ///
    /// The default cache capacity is 100 statements.
    pub fn statement_cache_capacity(mut self, capacity: usize) -> Self {
        self.statement_cache_capacity = capacity;
        self
    }
}

impl FromStr for SqliteConnectOptions {
    type Err = BoxDynError;

    fn from_str(mut s: &str) -> Result<Self, Self::Err> {
        let mut options = Self {
            filename: PathBuf::new(),
            in_memory: false,
            statement_cache_capacity: 100,
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
