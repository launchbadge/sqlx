//! **Pool** for SQLx database connections.

use std::{
    fmt,
    ops::{Deref, DerefMut},
    sync::Arc,
    time::{Duration, Instant},
};

use crate::connection::{Connect, Connection};

use self::inner::SharedPool;
pub use self::options::Builder;
use self::options::Options;

mod executor;
mod inner;
mod options;

/// A pool of database connections.
pub struct Pool<C>(Arc<SharedPool<C>>);

pub struct PoolConnection<C>
where
    C: Connection + Connect<Connection = C>,
{
    live: Option<Live<C>>,
    pool: Arc<SharedPool<C>>,
}

struct Live<C> {
    raw: C,
    created: Instant,
}

struct Idle<C> {
    live: Live<C>,
    since: Instant,
}

impl<C> Pool<C>
where
    C: Connection + Connect<Connection = C>,
{
    /// Creates a connection pool with the default configuration.
    ///
    /// The connection URL syntax is documented on the connection type for the respective
    /// database you're connecting to:
    ///
    /// * MySQL/MariaDB: [crate::MySqlConnection]
    /// * PostgreSQL: [crate::PgConnection]
    pub async fn new(url: &str) -> crate::Result<Self> {
        Self::builder().build(url).await
    }

    async fn with_options(url: &str, options: Options) -> crate::Result<Self> {
        let inner = SharedPool::new_arc(url, options).await?;

        Ok(Pool(inner))
    }

    /// Returns a [Builder] to configure a new connection pool.
    pub fn builder() -> Builder<C> {
        Builder::new()
    }

    /// Retrieves a connection from the pool.
    ///
    /// Waits for at most the configured connection timeout before returning an error.
    pub async fn acquire(&self) -> crate::Result<PoolConnection<C>> {
        self.0.acquire().await.map(|conn| PoolConnection {
            live: Some(conn),
            pool: Arc::clone(&self.0),
        })
    }

    /// Attempts to retrieve a connection from the pool if there is one available.
    ///
    /// Returns `None` immediately if there are no idle connections available in the pool.
    pub fn try_acquire(&self) -> Option<PoolConnection<C>> {
        self.0.try_acquire().map(|conn| PoolConnection {
            live: Some(conn),
            pool: Arc::clone(&self.0),
        })
    }

    /// Ends the use of a connection pool. Prevents any new connections
    /// and will close all active connections when they are returned to the pool.
    ///
    /// Does not resolve until all connections are closed.
    pub async fn close(&self) {
        self.0.close().await;
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
impl<C> Clone for Pool<C> {
    fn clone(&self) -> Self {
        Self(Arc::clone(&self.0))
    }
}

impl<C> fmt::Debug for Pool<C>
where
    C: Connection + Connect<Connection = C>,
{
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

const DEREF_ERR: &str = "(bug) connection already released to pool";

impl<C> Deref for PoolConnection<C>
where
    C: Connection + Connect<Connection = C>,
{
    type Target = C;

    fn deref(&self) -> &Self::Target {
        &self.live.as_ref().expect(DEREF_ERR).raw
    }
}

impl<C> DerefMut for PoolConnection<C>
where
    C: Connection + Connect<Connection = C>,
{
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.live.as_mut().expect(DEREF_ERR).raw
    }
}

impl<C> Drop for PoolConnection<C>
where
    C: Connection + Connect<Connection = C>,
{
    fn drop(&mut self) {
        if let Some(live) = self.live.take() {
            self.pool.release(live);
        }
    }
}
