//! **Pool** for SQLx database connections.

use std::{
    ops::{Deref, DerefMut},
    sync::Arc,
    time::{Duration, Instant},
};

use async_std::sync::Sender;
use futures_util::future::FutureExt;

use crate::Database;

use self::inner::SharedPool;
pub use self::options::Builder;
use self::options::Options;

mod executor;
mod inner;
mod options;

/// A pool of database connections.
pub struct Pool<DB>
where
    DB: Database,
{
    inner: Arc<SharedPool<DB>>,
    pool_tx: Sender<Idle<DB>>,
}

struct Connection<DB: Database> {
    raw: Option<Raw<DB>>,
    pool_tx: Sender<Idle<DB>>,
}

struct Raw<DB: Database> {
    inner: DB::Connection,
    created: Instant,
}

struct Idle<DB: Database> {
    raw: Raw<DB>,
    since: Instant,
}

impl<DB> Pool<DB>
where
    DB: Database,
    DB::Connection: crate::Connection<Database = DB>,
{
    /// Creates a connection pool with the default configuration.
    pub async fn new(url: &str) -> crate::Result<Self> {
        Self::with_options(url, Options::default()).await
    }

    async fn with_options(url: &str, options: Options) -> crate::Result<Self> {
        let (inner, pool_tx) = SharedPool::new_arc(url, options).await?;

        Ok(Pool { inner, pool_tx })
    }

    /// Returns a [Builder] to configure a new connection pool.
    pub fn builder() -> Builder<DB> {
        Builder::new()
    }

    /// Retrieves a connection from the pool.
    ///
    /// Waits for at most the configured connection timeout before returning an error.
    pub async fn acquire(&self) -> crate::Result<impl DerefMut<Target = DB::Connection>> {
        self.inner.acquire().await.map(|conn| Connection {
            raw: Some(conn),
            pool_tx: self.pool_tx.clone(),
        })
    }

    /// Attempts to retrieve a connection from the pool if there is one available.
    ///
    /// Returns `None` if there are no idle connections available in the pool.
    /// This method will not block waiting to establish a new connection.
    pub fn try_acquire(&self) -> Option<impl DerefMut<Target = DB::Connection>> {
        self.inner.try_acquire().map(|conn| Connection {
            raw: Some(conn),
            pool_tx: self.pool_tx.clone(),
        })
    }

    /// Ends the use of a connection pool. Prevents any new connections
    /// and will close all active connections when they are returned to the pool.
    ///
    /// Does not resolve until all connections are closed.
    pub async fn close(&self) {
        self.inner.close().await;
    }

    /// Returns the number of connections currently being managed by the pool.
    pub fn size(&self) -> u32 {
        self.inner.size()
    }

    /// Returns the number of idle connections.
    pub fn idle(&self) -> usize {
        self.inner.num_idle()
    }

    /// Returns the configured maximum pool size.
    pub fn max_size(&self) -> u32 {
        self.inner.options().max_size
    }

    /// Returns the maximum time spent acquiring a new connection before an error is returned.
    pub fn connect_timeout(&self) -> Duration {
        self.inner.options().connect_timeout
    }

    /// Returns the configured minimum idle connection count.
    pub fn min_idle(&self) -> u32 {
        self.inner.options().min_idle
    }

    /// Returns the configured maximum connection lifetime.
    pub fn max_lifetime(&self) -> Option<Duration> {
        self.inner.options().max_lifetime
    }

    /// Returns the configured idle connection timeout.
    pub fn idle_timeout(&self) -> Option<Duration> {
        self.inner.options().idle_timeout
    }
}

/// Returns a new [Pool] tied to the same shared connection pool.
impl<DB> Clone for Pool<DB>
where
    DB: Database,
{
    fn clone(&self) -> Self {
        Self {
            inner: Arc::clone(&self.inner),
            pool_tx: self.pool_tx.clone(),
        }
    }
}

const DEREF_ERR: &str = "(bug) connection already released to pool";

impl<DB: Database> Deref for Connection<DB> {
    type Target = DB::Connection;

    fn deref(&self) -> &Self::Target {
        &self.raw.as_ref().expect(DEREF_ERR).inner
    }
}

impl<DB: Database> DerefMut for Connection<DB> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.raw.as_mut().expect(DEREF_ERR).inner
    }
}

impl<DB: Database> Drop for Connection<DB> {
    fn drop(&mut self) {
        if let Some(conn) = self.raw.take() {
            self.pool_tx
                .send(Idle {
                    raw: conn,
                    since: Instant::now(),
                })
                .now_or_never()
                .expect("(bug) connection released into a full pool")
        }
    }
}
