use std::path::PathBuf;
use std::str::FromStr;

use crate::error::{BoxDynError, Error};

// TODO: Look at go-sqlite for option ideas
// TODO: journal_mode

#[derive(Debug)]
pub enum SqliteJournalMode {
    Delete,
    Truncate,
    Persist,
    Memory,
    Wal,
    Off,
}

impl SqliteJournalMode {
    pub(crate) fn as_str(&self) -> &'static str {
        match self {
            SqliteJournalMode::Delete => "DELETE",
            SqliteJournalMode::Truncate => "TRUNCATE",
            SqliteJournalMode::Persist => "PERSIST",
            SqliteJournalMode::Memory => "MEMORY",
            SqliteJournalMode::Wal => "WAL",
            SqliteJournalMode::Off => "OFF",
        }
    }
}

impl Default for SqliteJournalMode {
    fn default() -> Self {
        SqliteJournalMode::Wal
    }
}

impl FromStr for SqliteJournalMode {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Error> {
        Ok(match &*s.to_ascii_lowercase() {
            "delete" => SqliteJournalMode::Delete,
            "truncate" => SqliteJournalMode::Truncate,
            "persist" => SqliteJournalMode::Persist,
            "memory" => SqliteJournalMode::Memory,
            "wal" => SqliteJournalMode::Wal,
            "off" => SqliteJournalMode::Off,

            _ => {
                return Err(Error::ParseConnectOptions(
                    format!("unknown value {:?} for `journal_mode`", s).into(),
                ));
            }
        })
    }
}

/// Options and flags which can be used to configure a SQLite connection.
pub struct SqliteConnectOptions {
    pub(crate) filename: PathBuf,
    pub(crate) in_memory: bool,
    pub(crate) read_only: bool,
    pub(crate) journal_mode: SqliteJournalMode,
    pub(crate) foreign_keys: bool,
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
            read_only: false,
            foreign_keys: true,
            statement_cache_capacity: 100,
            journal_mode: SqliteJournalMode::Wal,
        }
    }

    /// Set the enforcement of [foreign key constriants](https://www.sqlite.org/pragma.html#pragma_foreign_keys).
    ///
    /// By default, this is enabled.
    pub fn foreign_keys(mut self, on: bool) -> Self {
        self.foreign_keys = on;
        self
    }

    /// Sets the [journal mode](https://www.sqlite.org/pragma.html#pragma_journal_mode) for the database connection.
    ///
    /// The default journal mode is WAL. For most use cases this can be significantly faster but
    /// there are [disadvantages](https://www.sqlite.org/wal.html).
    pub fn journal_mode(mut self, mode: SqliteJournalMode) -> Self {
        self.journal_mode = mode;
        self
    }

    /// Sets the [access mode](https://www.sqlite.org/c3ref/open.html) to open the database
    /// for read-only access.
    pub fn read_only(mut self, read_only: bool) -> Self {
        self.read_only = read_only;
        self
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
        let mut options = Self::new();

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
