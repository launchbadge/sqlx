use std::path::Path;

mod connect;
mod journal_mode;
mod parse;

pub use journal_mode::SqliteJournalMode;
use std::borrow::Cow;

/// Options and flags which can be used to configure a SQLite connection.
///
/// A value of `SqliteConnectOptions` can be parsed from a connection URI,
/// as described by [SQLite](https://www.sqlite.org/uri.html).
///
/// | URI | Description |
/// | -- | -- |
/// `sqlite::memory:` | Open an in-memory database. |
/// `sqlite:data.db` | Open the file `data.db` in the current directory. |
/// `sqlite://data.db` | Open the file `data.db` in the current directory. |
/// `sqlite:///data.db` | Open the file `data.db` from the root (`/`) directory. |
/// `sqlite://data.db?mode=ro` | Open the file `data.db` for read-only access. |
///
/// # Example
///
/// ```rust,no-run
/// # use sqlx_core as sqlx;
/// # use sqlx_core::connection::ConnectOptions;
/// # use sqlx_core::error::Error;
/// use sqlx::sqlite::{SqliteConnectOptions, SqliteJournalMode};
/// use std::str::FromStr;
///
/// # fn main() -> Result<(), Error> {
/// # #[cfg(feature = "runtime-async-std")]
/// # sqlx_rt::async_std::task::block_on(async move {
/// let conn = SqliteConnectOptions::from_str("sqlite://data.db")?
///     .journal_mode(SqliteJournalMode::Wal)
///     .read_only(true)
///     .connect().await?;
/// # Ok(())
/// # })
/// # }
/// ```
#[derive(Clone, Debug)]
pub struct SqliteConnectOptions {
    pub(crate) filename: Cow<'static, Path>,
    pub(crate) in_memory: bool,
    pub(crate) read_only: bool,
    pub(crate) create_if_missing: bool,
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
            filename: Cow::Borrowed(Path::new(":memory:")),
            in_memory: false,
            read_only: false,
            create_if_missing: false,
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

    /// Sets the [access mode](https://www.sqlite.org/c3ref/open.html) to create the database file
    /// if the file does not exist.
    ///
    /// By default, a new file **will be** created if one is not found.
    pub fn create_if_missing(mut self, create: bool) -> Self {
        self.create_if_missing = create;
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
