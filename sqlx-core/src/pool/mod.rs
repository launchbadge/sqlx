//! Connection pool for SQLx database connections.

use std::{
    fmt,
    sync::Arc,
    time::{Duration, Instant},
};

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

/// A pool of database connections.
pub struct Pool<DB: Database>(pub(crate) Arc<SharedPool<DB>>);

impl<DB: Database> Pool<DB> {
    /// Creates a connection pool with the default configuration.
    ///
    /// The connection URL syntax is documented on the connection type for the respective
    /// database you're connecting to:
    ///
    /// * MySQL/MariaDB: [crate::mysql::MySqlConnection]
    /// * PostgreSQL: [crate::postgres::PgConnection]
    pub async fn new(url: &str) -> Result<Self, Error> {
        Self::builder().build(url).await
    }

    async fn new_with(url: &str, options: Options) -> Result<Self, Error> {
        Ok(Pool(SharedPool::<DB>::new_arc(url, options).await?))
    }

    /// Returns a [`Builder`] to configure a new connection pool.
    pub fn builder() -> Builder<DB> {
        Builder::new()
    }

    /// Retrieves a connection from the pool.
    ///
    /// Waits for at most the configured connection timeout before returning an error.
    pub async fn acquire(&self) -> Result<PoolConnection<DB>, Error> {
        self.0.acquire().await.map(|conn| conn.attach(&self.0))
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
            .field("url", &self.0.url())
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

    fn assert_pool<C: Connect>() {
        assert_send_sync::<Pool<C>>();
        assert_clone::<Pool<C>>();
    }
}
