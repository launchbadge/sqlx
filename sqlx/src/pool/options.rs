use crate::pool::shared::SharedPool;
use crate::pool::Pool;
use crate::{Connect, ConnectOptions, Connection, Runtime};
use std::cmp;
use std::fmt::{self, Debug, Formatter};
use std::marker::PhantomData;
use std::sync::Arc;
use std::time::{Duration, Instant};

/// Configuration options/builder for constructing a [`Pool`].
///
/// See the source of [`Self::new()`] for the current defaults.
pub struct PoolOptions<Rt: Runtime, C: Connection<Rt>> {
    // general options
    pub(crate) max_connections: u32,
    pub(crate) acquire_timeout: Option<Duration>,
    pub(crate) min_connections: u32,
    pub(crate) max_lifetime: Option<Duration>,
    pub(crate) idle_timeout: Option<Duration>,

    // callback functions (any runtime)
    pub(crate) after_release: Option<Box<dyn Fn(&mut C) -> bool + 'static + Send + Sync>>,

    // callback functions (async)
    #[cfg(feature = "async")]
    pub(crate) after_connect_async: Option<
        Box<
            dyn Fn(&mut C) -> futures_util::BoxFuture<'_, crate::Result<()>>
                + Send
                + Sync
                + 'static,
        >,
    >,

    #[cfg(feature = "async")]
    pub(crate) before_acquire_async: Option<
        Box<
            dyn Fn(&mut C) -> futures_util::BoxFuture<'_, crate::Result<()>>
                + Send
                + Sync
                + 'static,
        >,
    >,

    //callback functions (blocking)
    #[cfg(feature = "blocking")]
    pub(crate) after_connect_blocking:
        Option<Box<dyn Fn(&mut C) -> crate::Result<()> + Send + Sync + 'static>>,
    #[cfg(feature = "blocking")]
    pub(crate) before_acquire_blocking:
        Option<Box<dyn Fn(&mut C) -> crate::Result<()> + Send + Sync + 'static>>,

    // to satisfy the orphan type params check
    _rt: PhantomData<Rt>,
}

impl<Rt: Runtime, C: Connection<Rt>> Default for PoolOptions<Rt, C> {
    fn default() -> Self {
        Self::new()
    }
}

impl<Rt: Runtime, C: Connection<Rt>> PoolOptions<Rt, C> {
    /// Create a new `PoolOptions` with some arbitrary, but sane, default values.
    ///
    /// See the source of this method for the current values.
    pub fn new() -> Self {
        Self {
            min_connections: 0,
            max_connections: 10,
            acquire_timeout: Some(Duration::from_secs(60)),
            idle_timeout: Some(Duration::from_secs(10 * 60)),
            max_lifetime: Some(Duration::from_secs(30 * 60)),
            after_release: None,
            #[cfg(feature = "async")]
            after_connect_async: None,
            #[cfg(feature = "async")]
            before_acquire_async: None,
            #[cfg(feature = "blocking")]
            after_connect_blocking: None,
            #[cfg(feature = "blocking")]
            before_acquire_blocking: None,
            _rt: PhantomData,
        }
        .test_before_acquire()
    }

    /// Set the minimum number of connections that this pool should maintain at all times.
    ///
    /// When the pool size drops below this amount, new connections are established automatically
    /// in the background.
    ///
    /// See the source of [`Self::new()`] for the default value.
    pub fn min_connections(mut self, min: u32) -> Self {
        self.min_connections = min;
        self
    }

    /// Set the maximum number of connections that this pool should maintain.
    ///
    /// See the source of [`Self::new()`] for the default value.
    pub fn max_connections(mut self, max: u32) -> Self {
        self.max_connections = max;
        self
    }

    /// Set the amount of time a task should wait while attempting to acquire a connection.
    ///
    /// If this timeout elapses, [`Pool::acquire()`] will return an error.
    ///
    /// If set to `None`, [`Pool::acquire()`] will wait as long as it takes to acquire
    /// a new connection.
    ///
    /// See the source of [`Self::new()`] for the default value.
    pub fn acquire_timeout(mut self, timeout: impl Into<Option<Duration>>) -> Self {
        self.acquire_timeout = timeout.into();
        self
    }

    /// Set the maximum lifetime of individual connections.
    ///
    /// Any connection with a lifetime greater than this will be closed.
    ///
    /// When set to `None`, all connections live until either reaped by [`idle_timeout`]
    /// or explicitly disconnected.
    ///
    /// Long-lived connections are not recommended due to the unfortunate reality of memory/resource
    /// leaks on the database-side. It is better to retire connections periodically
    /// (even if only once daily) to allow the database the opportunity to clean up data structures
    /// (parse trees, query metadata caches, thread-local storage, etc.) that are associated with a
    /// session.
    ///
    /// See the source of [`Self::new()`] for the default value.
    ///
    /// [`idle_timeout`]: Self::idle_timeout
    pub fn max_lifetime(mut self, lifetime: impl Into<Option<Duration>>) -> Self {
        self.max_lifetime = lifetime.into();
        self
    }

    /// Set a maximum idle duration for individual connections.
    ///
    /// Any connection with an idle duration longer than this will be closed.
    ///
    /// For usage-based database server billing, this can be a cost saver.
    ///
    /// See the source of [`Self::new()`] for the default value.
    pub fn idle_timeout(mut self, timeout: impl Into<Option<Duration>>) -> Self {
        self.idle_timeout = timeout.into();
        self
    }

    /// If enabled, the health of a connection will be verified by a call to [`Connection::ping`]
    /// before returning the connection.
    ///
    /// This overrides a previous callback set to [Self::before_acquire] and is also overridden by
    /// `before_acquire`.
    pub fn test_before_acquire(mut self) -> Self {
        #[cfg(feature = "async")]
        self.before_acquire_async = Some(Box::new(Connection::ping));
        #[cfg(feature = "blocking")]
        todo!("Connection doesn't have a ping_blocking()");

        self
    }

    pub fn after_release<F>(mut self, callback: F) -> Self
    where
        F: Fn(&mut C) -> bool + 'static + Send + Sync,
    {
        self.after_release = Some(Box::new(callback));
        self
    }

    /// Creates a new pool from this configuration.
    ///
    /// Note that **this does not immediately connect to the database**;
    /// this call will only error if the URI fails to parse.
    ///
    /// A connection will first be established either on the first call to
    /// [`Pool::acquire()`][super::Pool::acquire()] or,
    /// if [`self.min_connections`][Self::min_connections] is nonzero,
    /// when the background monitor task (async runtime) or thread (blocking runtime) is spawned.
    ///
    /// If you prefer to establish a minimum number of connections on startup to ensure a valid
    /// configuration, use [`.connect()`][Self::connect()] instead.
    ///
    /// See [`Self::build_with()`] for a version that lets you pass a [`ConnectOptions`].
    pub fn build(self, uri: &str) -> crate::Result<Pool<Rt, C>> {
        Ok(self.build_with(uri.parse()?))
    }

    /// Creates a new pool from this configuration.
    ///
    /// Note that **this does not immediately connect to the database**;
    /// this method call is infallible.
    ///
    /// A connection will first be established either on the first call to
    /// [`Pool::acquire()`][super::Pool::acquire()] or,
    /// if [`self.min_connections`][Self::min_connections] is nonzero,
    /// when the background monitor task (async runtime) or thread (blocking runtime) is spawned.
    ///
    /// If you prefer to establish at least one connections on startup to ensure a valid
    /// configuration, use [`.connect_with()`][Self::connect_with()] instead.
    pub fn build_with(self, options: <C as Connect<Rt>>::Options) -> Pool<Rt, C> {
        Pool { shared: SharedPool::new(self, options).into() }
    }
}

#[cfg(feature = "async")]
impl<Rt: crate::Async, C: Connection<Rt>> PoolOptions<Rt, C> {
    /// Perform an action after connecting to the database.
    pub fn after_connect<F>(mut self, callback: F) -> Self
    where
        for<'c> F:
            Fn(&'c mut C) -> futures_util::BoxFuture<'c, crate::Result<()>> + Send + Sync + 'static,
    {
        self.after_connect_async = Some(Box::new(callback));
        self
    }

    /// If set, this callback is executed with a connection that has been acquired from the idle
    /// queue.
    ///
    /// If the callback returns `Ok`, the acquired connection is returned to the caller. If
    /// it returns `Err`, the error is logged and the caller attempts to acquire another connection.
    ///
    /// This overrides [`Self::test_before_acquire()`].
    pub fn before_acquire<F>(mut self, callback: F) -> Self
    where
        for<'c> F: Fn(&'c mut C) -> futures_util::BoxFuture<'c, crate::Result<bool>>
            + Send
            + Sync
            + 'static,
    {
        self.before_acquire_async = Some(Box::new(callback));
        self
    }

    /// Creates a new pool from this configuration and immediately establishes
    /// [`self.min_connections`][Self::min_connections()],
    /// or just one connection if `min_connections == 0`.
    ///
    /// Returns an error if the URI fails to parse or an error occurs while establishing a connection.
    ///
    /// See [`Self::connect_with()`] for a version that lets you pass a [`ConnectOptions`].
    ///
    /// If you do not want to connect immediately on startup,
    /// use [`.build()`][Self::build()] instead.
    pub async fn connect(self, uri: &str) -> crate::Result<Pool<Rt, C>> {
        self.connect_with(uri.parse()?).await
    }

    /// Creates a new pool from this configuration and immediately establishes
    /// [`self.min_connections`][Self::min_connections()],
    /// or just one connection if `min_connections == 0`.
    ///
    /// Returns an error if an error occurs while establishing a connection.
    ///
    /// If you do not want to connect immediately on startup,
    /// use [`.build_with()`][Self::build_with()] instead.
    pub async fn connect_with(
        self,
        options: <C as Connect<Rt>>::Options,
    ) -> crate::Result<Pool<Rt, C>> {
        let mut shared = SharedPool::new(self, options);

        shared.init_min_connections_async().await?;

        Ok(Pool { shared: shared.into() })
    }
}

#[cfg(feature = "blocking")]
impl<C: Connection<crate::Blocking>> PoolOptions<crate::Blocking, C> {
    /// Perform an action after connecting to the database.
    pub fn after_connect(
        mut self,
        callback: impl Fn(&mut C) -> crate::Result<()> + Send + Sync + 'static,
    ) -> Self {
        self.after_connect_blocking = Some(Box::new(callback));
        self
    }

    /// If set, this callback is executed with a connection that has been acquired from the idle
    /// queue.
    ///
    /// If the callback returns `Ok`, the acquired connection is returned to the caller. If
    /// it returns `Err`, the error is logged and the caller attempts to acquire another connection.
    ///
    /// This overrides [`Self::test_before_acquire()`].
    pub fn before_acquire<F>(
        mut self,
        callback: impl Fn(&mut C) -> crate::Result<bool> + Send + Sync + 'static,
    ) -> Self {
        self.before_acquire_blocking = Some(Box::new(callback));
        self
    }

    /// Creates a new pool from this configuration and immediately establishes
    /// [`self.min_connections`][Self::min_connections()],
    /// or just one connection if `min_connections == 0`.
    ///
    /// Returns an error if the URI fails to parse or an error occurs while establishing a connection.
    ///
    /// See [`Self::connect_with()`] for a version that lets you pass a [`ConnectOptions`].
    ///
    /// If you do not want to connect immediately on startup,
    /// use [`.build()`][Self::build()] instead.
    pub fn connect(self, uri: &str) -> crate::Result<Pool<Rt, C>> {
        self.connect_with(uri.parse()?)
    }

    /// Creates a new pool from this configuration and immediately establishes
    /// [`self.min_connections`][Self::min_connections()],
    /// or just one connection if `min_connections == 0`.
    ///
    /// Returns an error if an error occurs while establishing a connection.
    ///
    /// If you do not want to connect immediately on startup,
    /// use [`.build_with()`][Self::build_with()] instead.
    pub fn connect_with(self, options: <C as Connect<Rt>>::Options) -> crate::Result<Pool<Rt, C>> {
        let mut shared = SharedPool::new(self, options);

        shared.init_min_connections_blocking()?;

        Ok(Pool { shared: shared.into() })
    }
}

impl<Rt: Runtime, C: Connection<Rt>> Debug for PoolOptions<Rt, C> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.debug_struct("PoolOptions")
            .field("max_connections", &self.max_connections)
            .field("min_connections", &self.min_connections)
            .field("acquire_timeout", &self.acquire_timeout)
            .field("max_lifetime", &self.max_lifetime)
            .field("idle_timeout", &self.idle_timeout)
            .finish()
    }
}
