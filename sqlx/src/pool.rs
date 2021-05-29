use crate::pool::connection::{Idle, Pooled};
use crate::pool::options::PoolOptions;
use crate::pool::shared::{SharedPool, TryAcquireResult};
use crate::pool::wait_list::WaitList;
use crate::{Connect, Connection, DefaultRuntime, Runtime};
use crossbeam_queue::ArrayQueue;
use std::sync::atomic::AtomicU32;
use std::sync::Arc;
use std::time::{Duration, Instant};

mod connection;
mod options;
mod shared;
mod wait_list;

/// SQLx's integrated, runtime-agnostic connection pool.
pub struct Pool<Rt: Runtime, C: Connection<Rt>> {
    shared: Arc<SharedPool<Rt, C>>,
}

impl<Rt: Runtime, C: Connection<Rt>> Pool<Rt, C> {
    /// Construct a new pool with default configuration and given connection URI.
    ///
    /// Connection will not be attempted until the first call to [`Self::acquire()`].
    /// The only error that may be returned is [`Error::ConnectOptions`][crate::Error::ConnectOptions]
    /// if the passed URI fails to parse or contains invalid options.
    ///
    /// If you want to eagerly connect on construction of the pool, use [`Self::connect`]
    /// instead.
    ///
    /// See also:
    /// * [`Self::new_with()`]
    /// * [`Self::builder()`] (alias of [`PoolOptions::new()`]) and [`PoolOptions::build()`]
    pub fn new(uri: &str) -> crate::Result<Self> {
        Self::builder().build(uri)
    }

    /// Construct a new pool with default configuration and given connection options.
    ///    
    /// Connection will not be attempted until the first call to [`Self::acquire()`].
    /// The only error that may be returned is [`Error::ConnectOptions`][crate::Error::ConnectOptions]
    /// if the passed URI fails to parse or contains invalid options.
    ///
    /// If you want to eagerly connect on construction of the pool, use [`Self::connect_with()`]
    /// instead.
    ///
    /// See also:
    /// * [`Self::builder()`] (alias of [`PoolOptions::new()`]) and [`PoolOptions::build_with()`]
    pub fn new_with(connect_options: <C as Connect<Rt>>::Options) -> Self {
        Self::builder().build_with(connect_options)
    }

    /// A helpful alias for [`PoolOptions::new()`].
    pub fn builder() -> PoolOptions<Rt, C> {
        PoolOptions::new()
    }
}

#[cfg(feature = "async")]
impl<Rt: crate::Async, C: Connection<Rt>> Pool<Rt, C> {
    /// Construct a new pool with default configuration and given connection URI
    /// and establish at least one connection to ensure the URI is valid.
    ///
    /// See also:
    /// * [`Self::connect_with()`]
    /// * [`Self::builder()`] (alias of [`PoolOptions::new()`]) and [`PoolOptions::connect()`]
    pub async fn connect(uri: &str) -> crate::Result<Self> {
        Self::builder().connect(uri).await
    }

    /// Construct a new pool with default configuration and given connection options
    /// and establish at least one connection to ensure the latter are valid.
    ///
    /// See also:
    /// * [`Self::builder()`] (alias of [`PoolOptions::new()`]) and [`PoolOptions::connect_with()`]
    pub async fn connect_with(connect_options: <C as Connect<Rt>>::Options) -> crate::Result<Self> {
        Self::builder().connect_with(connect_options).await
    }

    /// Acquire a connection from the pool.
    ///
    /// This will either wait until a connection is released by another task (via the `Drop` impl on
    /// [`Pooled`]) or, if the pool is not yet at its maximum size, opening a new connection.
    ///
    /// If an acquire timeout is configured via [`PoolOptions::acquire_timeout()`], this will wait
    /// at most the given duration before returning [`Error::AcquireTimedOut`][crate::Error::AcquireTimedOut].
    ///
    /// See also:
    /// * [`Self::acquire_timeout()`]
    /// * [`PoolOptions::max_connections()`]
    /// * [`PoolOptions::acquire_timeout()`]
    pub async fn acquire(&self) -> crate::Result<Pooled<Rt, C>> {
        if let Some(timeout) = self.shared.pool_options.acquire_timeout {
            self.acquire_timeout(timeout).await
        } else {
            self.acquire_inner().await
        }
    }

    /// Acquire a connection from the pool, waiting at most the given duration.
    ///
    /// This will either wait until a connection is released by another task (via the `Drop` impl on
    /// [`Pooled`]) or, if the pool is not yet at its maximum size, opening a new connection.
    ///
    /// If the given duration elapses, this will return
    /// [`Error::AcquireTimedOut`][crate::Error::AcquireTimedOut].
    pub async fn acquire_timeout(&self, timeout: Duration) -> crate::Result<Pooled<Rt, C>> {
        Rt::timeout_async(timeout, self.acquire_inner())
            .await
            .ok_or(crate::Error::AcquireTimedOut)?
    }

    async fn acquire_inner(&self) -> crate::Result<Pooled<Rt, C>> {
        let mut acquire_permit = None;

        loop {
            match self.shared.try_acquire(acquire_permit.take()) {
                TryAcquireResult::Acquired(mut conn) => {
                    match self.shared.on_acquire_async(&mut conn).await {
                        Ok(()) => return Ok(conn.attach(&self.shared)),
                        Err(e) => {
                            log::info!("error from before_acquire: {:?}", e);
                        }
                    }
                }
                TryAcquireResult::Connect(permit) => self.shared.connect_async(permit).await,
                TryAcquireResult::Wait => {
                    acquire_permit = Some(self.shared.wait_async().await);
                }
                TryAcquireResult::PoolClosed => Err(crate::Error::Closed),
            }
        }
    }
}

#[cfg(feature = "blocking")]
impl<C: Connection<crate::Blocking>> Pool<crate::Blocking, C> {
    /// Construct a new pool with default configuration and given connection URI
    /// and establish at least one connection to ensure the URI is valid.
    ///
    /// See also:
    /// * [`Self::connect_with()`]
    /// * [`Self::builder()`] (alias of [`PoolOptions::new()`]) and [`PoolOptions::connect()`]
    pub fn connect(uri: &str) -> crate::Result<Self> {
        Self::builder().connect(uri)
    }

    /// Construct a new pool with default configuration and given connection options
    /// and establish at least one connection to ensure the latter are valid.
    ///
    /// See also:
    /// * [`Self::builder()`] (alias of [`PoolOptions::new()`]) and [`PoolOptions::connect_with()`]
    pub fn connect_with(
        connect_options: <C as Connect<crate::Blocking>>::Options,
    ) -> crate::Result<Self> {
        Self::builder().connect_with(connect_options)
    }

    /// Acquire a connection from the pool.
    ///
    /// This will either wait until a connection is released by another thread (via the `Drop` impl on
    /// [`Pooled`]) or, if the pool is not yet at its maximum size, opening a new connection.
    ///
    /// If an acquire timeout is configured via [`PoolOptions::acquire_timeout()`], this will wait
    /// at most the given duration before returning [`Error::AcquireTimedOut`][crate::Error::AcquireTimedOut].
    ///
    /// See also:
    /// * [`Self::acquire_timeout()`]
    /// * [`PoolOptions::max_connections()`]
    /// * [`PoolOptions::acquire_timeout()`]
    pub fn acquire(&self) -> crate::Result<Pooled<crate::Blocking, C>> {
        self.acquire_inner(self.shared.pool_options.acquire_timeout)
    }

    /// Acquire a connection from the pool, waiting at most the given duration.
    ///
    /// This will either wait until a connection is released by another thread (via the `Drop` impl on
    /// [`Pooled`]) or, if the pool is not yet at its maximum size, opening a new connection.
    ///
    /// If the given duration elapses, this will return
    /// [`Error::AcquireTimedOut`][crate::Error::AcquireTimedOut].
    pub fn acquire_timeout(&self, timeout: Duration) -> crate::Result<Pooled<crate::Blocking, C>> {
        self.acquire_inner(Some(timeout))
    }

    fn acquire_inner(
        &self,
        timeout: Option<Duration>,
    ) -> crate::Result<Pooled<crate::Blocking, C>> {
        let mut acquire_permit = None;

        let deadline = timeout.map(|timeout| Instant::now() + timeout);

        loop {
            match self.shared.try_acquire(acquire_permit.take()) {
                TryAcquireResult::Acquired(mut conn) => {
                    match self.shared.on_acquire_blocking(&mut conn) {
                        Ok(()) => return Ok(conn.attach(&self.shared)),
                        Err(e) => {
                            log::info!("error from before_acquire: {:?}", e);
                        }
                    }
                }
                TryAcquireResult::Connect(permit) => self.shared.connect_blocking(permit),
                TryAcquireResult::Wait => {
                    acquire_permit = Some(
                        self.shared.wait_blocking(deadline).ok_or(crate::Error::AcquireTimedOut)?,
                    );
                }
                TryAcquireResult::PoolClosed => Err(crate::Error::Closed),
            }
        }
    }
}
