use std::path::Path;

mod auto_vacuum;
mod connect;
mod journal_mode;
mod locking_mode;
mod parse;
mod synchronous;

use crate::connection::LogSettings;
pub use auto_vacuum::SqliteAutoVacuum;
pub use journal_mode::SqliteJournalMode;
pub use locking_mode::SqliteLockingMode;
use std::{borrow::Cow, time::Duration};
pub use synchronous::SqliteSynchronous;

use indexmap::IndexMap;

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
/// ```rust,no_run
/// # use sqlx_core as sqlx;
/// # use sqlx_core::connection::ConnectOptions;
/// # use sqlx_core::error::Error;
/// use sqlx::sqlite::{SqliteConnectOptions, SqliteJournalMode};
/// use std::str::FromStr;
///
/// # fn main() {
/// # #[cfg(feature = "_rt-async-std")]
/// # sqlx_rt::async_std::task::block_on::<_, Result<(), Error>>(async move {
/// let conn = SqliteConnectOptions::from_str("sqlite://data.db")?
///     .journal_mode(SqliteJournalMode::Wal)
///     .read_only(true)
///     .connect().await?;
/// # Ok(())
/// # }).unwrap();
/// # }
/// ```
#[derive(Clone, Debug)]
pub struct SqliteConnectOptions {
    pub(crate) filename: Cow<'static, Path>,
    pub(crate) in_memory: bool,
    pub(crate) read_only: bool,
    pub(crate) create_if_missing: bool,
    pub(crate) shared_cache: bool,
    pub(crate) statement_cache_capacity: usize,
    pub(crate) busy_timeout: Duration,
    pub(crate) log_settings: LogSettings,
    pub(crate) immutable: bool,
    pub(crate) pragmas: IndexMap<Cow<'static, str>, Cow<'static, str>>,
    pub(crate) serialized: bool,
}

impl Default for SqliteConnectOptions {
    fn default() -> Self {
        Self::new()
    }
}

impl SqliteConnectOptions {
    pub fn new() -> Self {
        // set default pragmas
        let mut pragmas: IndexMap<Cow<'static, str>, Cow<'static, str>> = IndexMap::new();

        let locking_mode: SqliteLockingMode = Default::default();
        let auto_vacuum: SqliteAutoVacuum = Default::default();

        // page_size must be set before any other action on the database.
        pragmas.insert("page_size".into(), "4096".into());

        // Note that locking_mode should be set before journal_mode; see
        // https://www.sqlite.org/wal.html#use_of_wal_without_shared_memory .
        pragmas.insert("locking_mode".into(), locking_mode.as_str().into());

        pragmas.insert(
            "journal_mode".into(),
            SqliteJournalMode::Wal.as_str().into(),
        );

        pragmas.insert("foreign_keys".into(), "ON".into());

        pragmas.insert(
            "synchronous".into(),
            SqliteSynchronous::Full.as_str().into(),
        );

        pragmas.insert("auto_vacuum".into(), auto_vacuum.as_str().into());

        Self {
            filename: Cow::Borrowed(Path::new(":memory:")),
            in_memory: false,
            read_only: false,
            create_if_missing: false,
            shared_cache: false,
            statement_cache_capacity: 100,
            busy_timeout: Duration::from_secs(5),
            log_settings: Default::default(),
            immutable: false,
            pragmas,
            serialized: false,
        }
    }

    /// Sets the name of the database file.
    pub fn filename(mut self, filename: impl AsRef<Path>) -> Self {
        self.filename = Cow::Owned(filename.as_ref().to_owned());
        self
    }

    /// Set the enforcement of [foreign key constriants](https://www.sqlite.org/pragma.html#pragma_foreign_keys).
    ///
    /// By default, this is enabled.
    pub fn foreign_keys(mut self, on: bool) -> Self {
        self.pragmas.insert(
            "foreign_keys".into(),
            (if on { "ON" } else { "OFF" }).into(),
        );
        self
    }

    /// Set the [`SQLITE_OPEN_SHAREDCACHE` flag](https://sqlite.org/sharedcache.html).
    ///
    /// By default, this is disabled.
    pub fn shared_cache(mut self, on: bool) -> Self {
        self.shared_cache = on;
        self
    }

    /// Sets the [journal mode](https://www.sqlite.org/pragma.html#pragma_journal_mode) for the database connection.
    ///
    /// The default journal mode is WAL. For most use cases this can be significantly faster but
    /// there are [disadvantages](https://www.sqlite.org/wal.html).
    pub fn journal_mode(mut self, mode: SqliteJournalMode) -> Self {
        self.pragmas
            .insert("journal_mode".into(), mode.as_str().into());
        self
    }

    /// Sets the [locking mode](https://www.sqlite.org/pragma.html#pragma_locking_mode) for the database connection.
    ///
    /// The default locking mode is NORMAL.
    pub fn locking_mode(mut self, mode: SqliteLockingMode) -> Self {
        self.pragmas
            .insert("locking_mode".into(), mode.as_str().into());
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
    /// By default, a new file **will not be** created if one is not found.
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

    /// Sets a timeout value to wait when the database is locked, before
    /// returning a busy timeout error.
    ///
    /// The default busy timeout is 5 seconds.
    pub fn busy_timeout(mut self, timeout: Duration) -> Self {
        self.busy_timeout = timeout;
        self
    }

    /// Sets the [synchronous](https://www.sqlite.org/pragma.html#pragma_synchronous) setting for the database connection.
    ///
    /// The default synchronous settings is FULL. However, if durability is not a concern,
    /// then NORMAL is normally all one needs in WAL mode.
    pub fn synchronous(mut self, synchronous: SqliteSynchronous) -> Self {
        self.pragmas
            .insert("synchronous".into(), synchronous.as_str().into());
        self
    }

    /// Sets the [auto_vacuum](https://www.sqlite.org/pragma.html#pragma_auto_vacuum) setting for the database connection.
    ///
    /// The default auto_vacuum setting is NONE.
    pub fn auto_vacuum(mut self, auto_vacuum: SqliteAutoVacuum) -> Self {
        self.pragmas
            .insert("auto_vacuum".into(), auto_vacuum.as_str().into());
        self
    }

    /// Sets the [page_size](https://www.sqlite.org/pragma.html#pragma_page_size) setting for the database connection.
    ///
    /// The default page_size setting is 4096.
    pub fn page_size(mut self, page_size: u32) -> Self {
        self.pragmas
            .insert("page_size".into(), page_size.to_string().into());
        self
    }

    /// Sets custom initial pragma for the database connection.
    pub fn pragma<K, V>(mut self, key: K, value: V) -> Self
    where
        K: Into<Cow<'static, str>>,
        V: Into<Cow<'static, str>>,
    {
        self.pragmas.insert(key.into(), value.into());
        self
    }

    pub fn immutable(mut self, immutable: bool) -> Self {
        self.immutable = immutable;
        self
    }

    /// Sets the [threading mode](https://www.sqlite.org/threadsafe.html) for the database connection.
    ///
    /// The default setting is `false` corersponding to using `OPEN_NOMUTEX`, if `true` then `OPEN_FULLMUTEX`.
    ///
    /// See [open](https://www.sqlite.org/c3ref/open.html) for more details.
    pub fn serialized(mut self, serialized: bool) -> Self {
        self.serialized = serialized;
        self
    }
}
