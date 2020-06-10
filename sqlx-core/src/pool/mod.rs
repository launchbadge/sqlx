//! Provides the connection pool for asynchronous SQLx connections.
//!
//! Opening a database connection for each and every operation to the database can quickly
//! become expensive. Furthermore, sharing a database connection between threads and functions
//! can be difficult to express in Rust.
//!
//! A connection pool is a standard technique that can manage opening and re-using connections.
//! Normally it also enforces a maximum number of connections as these are an expensive resource
//! on the database server.
//!
//! SQLx provides a canonical connection pool implementation intended to satisfy the majority
//! of use cases.
//!
//! # Opening a connection pool
//!
//! A new connection pool with a default configuration can be created by supplying `Pool`
//! with the database driver and a connection string.
//!
//! ```rust,ignore
//! use sqlx::Pool;
//! use sqlx::postgres::Postgres;
//!
//! let pool = Pool::<Postgres>::new("postgres://").await?;
//! ```
//!
//! For convenience, database-specific type aliases are provided:
//!
//! ```rust,ignore
//! use sqlx::mssql::MssqlPool;
//!
//! let pool = MssqlPool::new("mssql://").await?;
//! ```
//!
//! # Using a connection pool
//!
//! A connection pool implements [`Executor`](../trait.Executor.html) and can be used directly
//! when executing a query. Notice that only an immutable reference (`&Pool`) is needed.
//!
//! ```rust,ignore
//! sqlx::query("DELETE FROM articles").execute(&pool).await?;
//! ```
//!
//! A connection or transaction may also be manually acquired with
//! [`Pool::acquire`](struct.Pool.html#method.acquire) or
//! [`Pool::begin`](struct.Pool.html#method.begin).
//!

use std::fmt;
use std::future::Future;
use std::sync::Arc;
use std::time::{Duration, Instant};

use crate::connection::Connect;
use crate::database::Database;
use crate::error::Error;
use crate::transaction::Transaction;

use self::inner::SharedPool;
use self::options::Options;

#[macro_use]
mod executor;

mod connection;
mod inner;
mod options;

pub use self::connection::PoolConnection;
pub use self::options::Builder;

/// An alias for [`Transaction`] when returned from [`Pool::begin`].
pub type PoolTransaction<DB, C = PoolConnection<DB>> = Transaction<'static, DB, C>;

/// An asynchronous pool of SQLx database connections.
pub struct Pool<DB: Database>(pub(crate) Arc<SharedPool<DB>>);

impl<DB: Database> Pool<DB> {
    /// Creates a new connection pool with the default pool configuration and the given connection
    /// string.
    ///
    /// The connection string is parsed according to the connection options for
    /// the current database.
    pub async fn new(url: &str) -> Result<Self, Error> {
        Self::builder().build(url).await
    }

    /// Creates a new connection pool with the default pool configuration and the given connection
    /// options.
    pub async fn new_with(options: <DB::Connection as Connect>::Options) -> Result<Self, Error> {
        Self::builder().build_with(options).await
    }

    /// Returns a [`Builder`] to configure a new connection pool.
    pub fn builder() -> Builder<DB> {
        Builder::new()
    }

    /// Retrieves a connection from the pool.
    ///
    /// Waits for at most the configured connection timeout before returning an error.
    pub fn acquire(&self) -> impl Future<Output = Result<PoolConnection<DB>, Error>> + 'static {
        let shared = self.0.clone();
        async move { shared.acquire().await.map(|conn| conn.attach(&shared)) }
    }

    /// Attempts to retrieve a connection from the pool if there is one available.
    ///
    /// Returns `None` immediately if there are no idle connections available in the pool.
    pub fn try_acquire(&self) -> Option<PoolConnection<DB>> {
        self.0.try_acquire().map(|conn| conn.attach(&self.0))
    }

    /// Retrieves a new connection and immediately begins a new transaction.
    pub async fn begin(&self) -> Result<Transaction<'static, DB, PoolConnection<DB>>, Error> {
        Ok(Transaction::begin(self.acquire().await?).await?)
    }

    /// Attempts to retrieve a new connection and immediately begins a new transaction if there
    /// is one available.
    pub async fn try_begin(
        &self,
    ) -> Result<Option<Transaction<'static, DB, PoolConnection<DB>>>, Error> {
        match self.try_acquire() {
            Some(conn) => Transaction::begin(conn).await.map(Some),
            None => Ok(None),
        }
    }

    /// Ends the use of a connection pool. Prevents any new connections
    /// and will close all active connections when they are returned to the pool.
    ///
    /// Does not resolve until all connections are closed.
    pub async fn close(&self) {
        self.0.close().await;
    }

    /// Returns `true` if [`.close()`][Pool::close] has been called on the pool, `false` otherwise.
    pub fn is_closed(&self) -> bool {
        self.0.is_closed()
    }

    /// Returns the number of connections currently being managed by the pool.
    pub fn size(&self) -> u32 {
        self.0.size()
    }

    /// Returns the number of idle connections.
    pub fn idle(&self) -> usize {
        self.0.num_idle()
    }

    /// Returns the configured maximum pool size.
    pub fn max_size(&self) -> u32 {
        self.0.options().max_size
    }

    /// Returns the maximum time spent acquiring a new connection before an error is returned.
    pub fn connect_timeout(&self) -> Duration {
        self.0.options().connect_timeout
    }

    /// Returns the configured minimum idle connection count.
    pub fn min_size(&self) -> u32 {
        self.0.options().min_size
    }

    /// Returns the configured maximum connection lifetime.
    pub fn max_lifetime(&self) -> Option<Duration> {
        self.0.options().max_lifetime
    }

    /// Returns the configured idle connection timeout.
    pub fn idle_timeout(&self) -> Option<Duration> {
        self.0.options().idle_timeout
    }
}

/// Returns a new [Pool] tied to the same shared connection pool.
impl<DB: Database> Clone for Pool<DB> {
    fn clone(&self) -> Self {
        Self(Arc::clone(&self.0))
    }
}

impl<DB: Database> fmt::Debug for Pool<DB> {
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt.debug_struct("Pool")
            .field("size", &self.0.size())
            .field("num_idle", &self.0.num_idle())
            .field("is_closed", &self.0.is_closed())
            .field("options", self.0.options())
            .finish()
    }
}

/// get the time between the deadline and now and use that as our timeout
///
/// returns `Error::PoolTimedOut` if the deadline is in the past
fn deadline_as_timeout<DB: Database>(deadline: Instant) -> Result<Duration, Error> {
    deadline
        .checked_duration_since(Instant::now())
        .ok_or(Error::PoolTimedOut)
}

#[test]
#[allow(dead_code)]
fn assert_pool_traits() {
    fn assert_send_sync<T: Send + Sync>() {}
    fn assert_clone<T: Clone>() {}

    fn assert_pool<DB: Database>() {
        assert_send_sync::<Pool<DB>>();
        assert_clone::<Pool<DB>>();
    }
}
