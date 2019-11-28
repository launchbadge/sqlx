use crate::{
    backend::Backend, connection::Connection, error::Error, executor::Executor,
    params::IntoQueryParameters, row::FromRow, Row,
};
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

pub(crate) use self::inner::{Live, SharedPool};
use self::options::Options;

pub use self::options::Builder;

mod executor;
mod inner;
mod options;

/// A pool of database connections.
pub struct Pool<DB>(Arc<SharedPool<DB>>)
where
    DB: Backend;

impl<DB> Pool<DB>
where
    DB: Backend,
{
    /// Creates a connection pool with the default configuration.
    pub async fn new(url: &str) -> crate::Result<Self> {
        Self::with_options(url, Options::default()).await
    }

    async fn with_options(url: &str, options: Options) -> crate::Result<Self> {
        Ok(Pool(SharedPool::new_arc(url, options).await?))
    }

    /// Returns a [Builder] to configure a new connection pool.
    pub fn builder() -> Builder<DB> {
        Builder::new()
    }

    /// Retrieves a connection from the pool.
    ///
    /// Waits for at most the configured connection timeout before returning an error.
    pub async fn acquire(&self) -> crate::Result<Connection<DB>> {
        self.0.acquire().await.map(Connection::new)
    }

    /// Attempts to retrieve a connection from the pool if there is one available.
    ///
    /// Returns `None` if there are no idle connections available in the pool.
    /// This method will not block waiting to establish a new connection.
    pub fn try_acquire(&self) -> Option<Connection<DB>> {
        self.0.try_acquire().map(Connection::new)
    }

    /// Ends the use of a connection pool. Prevents any new connections
    /// and will close all active connections when they are returned to the pool.
    ///
    /// Does not resolve until all connections are closed.
    pub async fn close(&self) {
        let _ = self.0.close().await;
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
    pub fn min_idle(&self) -> u32 {
        self.0.options().min_idle
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
impl<DB> Clone for Pool<DB>
where
    DB: Backend,
{
    fn clone(&self) -> Self {
        Self(Arc::clone(&self.0))
    }
}
