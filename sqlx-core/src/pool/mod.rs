use crate::{backend::Backend, error::Error, executor::Executor, params::IntoQueryParameters, row::FromRow, Row};
use futures_channel::oneshot;
use futures_core::{future::BoxFuture, stream::BoxStream};
use futures_util::{
    future::{AbortHandle, AbortRegistration, FutureExt, TryFutureExt},
    stream::StreamExt,
};
use std::{
    cmp,
    future::Future,
    marker::PhantomData,
    ops::{Deref, DerefMut},
    sync::{
        atomic::{AtomicBool, AtomicU32, AtomicUsize, Ordering},
        Arc,
    },
    time::{Duration, Instant},
};

use async_std::{
    future::timeout,
    sync::{channel, Receiver, Sender},
    task,
};

use self::inner::SharedPool;

use self::options::Options;

pub use self::options::Builder;

mod executor;
mod inner;
mod options;

/// A pool of database connections.
pub struct Pool<DB>
    where
        DB: Backend
{
    inner: Arc<SharedPool<DB>>,
    pool_tx: Sender<Idle<DB>>
}

/// A connection tied to a pool. When dropped it is released back to the pool.
pub struct Connection<DB: Backend> {
    raw: Option<Raw<DB>>,
    pool_tx: Sender<Idle<DB>>,
}

struct Raw<DB: Backend> {
    inner: DB::Connection,
    created: Instant,
}

struct Idle<DB: Backend> {
    raw: Raw<DB>,
    since: Instant,
}

impl<DB> Pool<DB>
where
    DB: Backend, DB::Connection: crate::Connection<Backend = DB>
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
    pub async fn acquire(&self) -> crate::Result<Connection<DB>> {
        self.inner.acquire().await.map(|conn| Connection { raw: Some(conn), pool_tx: self.pool_tx.clone() })
    }

    /// Attempts to retrieve a connection from the pool if there is one available.
    ///
    /// Returns `None` if there are no idle connections available in the pool.
    /// This method will not block waiting to establish a new connection.
    pub fn try_acquire(&self) -> Option<Connection<DB>> {
        self.inner.try_acquire().map(|conn| Connection { raw: Some(conn), pool_tx: self.pool_tx.clone() })
    }

    /// Ends the use of a connection pool. Prevents any new connections
    /// and will close all active connections when they are returned to the pool.
    ///
    /// Does not resolve until all connections are closed.
    pub async fn close(&self) {
        let _ = self.inner.close().await;
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
    DB: Backend,
{
    fn clone(&self) -> Self {
        Self {
            inner: Arc::clone(&self.inner),
            pool_tx: self.pool_tx.clone(),
        }
    }
}

const DEREF_ERR: &str = "(bug) connection already released to pool";

impl<DB: Backend> Deref for Connection<DB> {
    type Target = DB::Connection;

    fn deref(&self) -> &Self::Target {
        &self.raw.as_ref().expect(DEREF_ERR).inner
    }
}

impl<DB: Backend> DerefMut for Connection<DB> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.raw.as_mut().expect(DEREF_ERR).inner
    }
}

impl<DB: Backend> Drop for Connection<DB> {
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
