use crate::connection::{ConnectOptions, Connection};
use crate::database::Database;
use crate::pool::connection::Floating;
use crate::pool::inner::PoolInner;
use crate::pool::PoolConnection;
use crate::rt::JoinHandle;
use crate::Error;
use ease_off::EaseOff;
use event_listener::Event;
use std::fmt::{Display, Formatter};
use std::future::Future;
use std::ptr;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use std::time::Instant;

use std::io;

/// Custom connect callback for [`Pool`][crate::pool::Pool].
///
/// Implemented for closures with the signature
/// `Fn(PoolConnectMetadata) -> impl Future<Output = sqlx::Result<impl Connection>>`.
///
/// See [`Self::connect()`] for details and implementation advice.
///
/// # Example: `after_connect` Replacement
/// The `after_connect` callback was removed in 0.9.0 as it was redundant to this API.
///
/// This example uses Postgres but may be adapted to any driver.
///
/// ```rust,no_run
/// use std::sync::Arc;
/// use sqlx::PgConnection;
/// use sqlx::postgres::PgPoolOptions;
/// use sqlx::Connection;
///
/// # async fn _example() -> sqlx::Result<()> {
/// // `PoolConnector` is implemented for closures but has restrictions on returning borrows
/// // due to current language limitations.
/// //
/// // This example shows how to get around this using `Arc`.
/// let database_url: Arc<str> = "postgres://...".into();
///
/// let pool = PgPoolOptions::new()
///     .min_connections(5)
///     .max_connections(30)
///     .connect_with_connector(move |meta| {
///         let database_url = database_url.clone();
///         async move {
///             println!(
///                 "opening connection {}, attempt {}; elapsed time: {}",
///                 meta.pool_size,
///                 meta.num_attempts + 1,
///                 meta.start.elapsed()
///             );
///
///             let mut conn = PgConnection::connect(&database_url).await?;
///
///             // Override the time zone of the connection.
///             sqlx::raw_sql("SET TIME ZONE 'Europe/Berlin'").await?;
///
///             Ok(conn)
///         }
///     })
///     .await?;
/// # Ok(())
/// # }
/// ```
///
/// # Example: `set_connect_options` Replacement
/// `set_connect_options` and `get_connect_options` were removed in 0.9.0 because they complicated
/// the pool internals. They can be reimplemented by capturing a mutex, or similar, in the callback.
///
/// This example uses Postgres and [`tokio::sync::RwLock`] but may be adapted to any driver
/// or `async-std`, respectively.
///
/// ```rust,no_run
/// use std::sync::Arc;
/// use tokio::sync::{Mutex, RwLock};
/// use sqlx::PgConnection;
/// use sqlx::postgres::PgConnectOptions;
/// use sqlx::postgres::PgPoolOptions;
/// use sqlx::ConnectOptions;
///
/// # async fn _example() -> sqlx::Result<()> {
/// // If you do not wish to hold the lock during the connection attempt,
/// // you could use `Arc<PgConnectOptions>` instead.
/// let connect_opts: Arc<RwLock<PgConnectOptions>> = Arc::new(RwLock::new("postgres://...".parse()?));
/// // We need a copy that will be captured by the closure.
/// let connect_opts_ = connect_opts.clone();
///
/// let pool = PgPoolOptions::new()
///     .connect_with_connector(move |meta| {
///         let connect_opts_ = connect_opts.clone();
///         async move {
///             println!(
///                 "opening connection {}, attempt {}; elapsed time: {}",
///                 meta.pool_size,
///                 meta.num_attempts + 1,
///                 meta.start.elapsed()
///             );
///
///             connect_opts.read().await.connect().await
///         }
///     })
///     .await?;
///
/// // Close the connection that was previously opened by `connect_with_connector()`.
/// pool.acquire().await?.close().await?;
///
/// // Simulating a credential rotation
/// let mut write_connect_opts = connect_opts.write().await;
/// write_connect_opts
///     .set_username("new_username")
///     .set_password("new password");
///
/// // Should use the new credentials.
/// let mut conn = pool.acquire().await?;
///
/// # Ok(())
/// # }
/// ```
///
/// # Example: Custom Implementation
///
/// Custom implementations of `PoolConnector` trade a little bit of boilerplate for much
/// more flexibility. Thanks to the signature of `connect()`, they can return a `Future`
/// type that borrows from `self`.
///
/// This example uses Postgres but may be adapted to any driver.
///
/// ```rust,no_run
/// use sqlx::{PgConnection, Postgres};
/// use sqlx::postgres::{PgConnectOptions, PgPoolOptions};
/// use sqlx_core::connection::ConnectOptions;
/// use sqlx_core::pool::{PoolConnectMetadata, PoolConnector};
///
/// struct MyConnector {
///     // A list of servers to connect to in a high-availability configuration.
///     host_ports: Vec<(String, u16)>,
///     username: String,
///     password: String,
/// }
///
/// impl PoolConnector<Postgres> for MyConnector {
///     // The desugaring of `async fn` is compatible with the signature of `connect()`.
///     async fn connect(&self, meta: PoolConnectMetadata) -> sqlx::Result<PgConnection> {
///         self.get_connect_options(meta.num_attempts)
///             .connect()
///             .await
///     }
/// }
///
/// impl MyConnector {
///     fn get_connect_options(&self, attempt: usize) -> PgConnectOptions {
///         // Select servers in a round-robin.
///         let (ref host, port) = self.host_ports[attempt % self.host_ports.len()];
///
///         PgConnectOptions::new()
///             .host(host)
///             .port(port)
///             .username(&self.username)
///             .password(&self.password)
///     }
/// }
///
/// # async fn _example() -> sqlx::Result<()> {
/// let pool = PgPoolOptions::new()
///     .max_connections(25)
///     .connect_with_connector(MyConnector {
///         host_ports: vec![
///             ("db1.postgres.cluster.local".into(), 5432),
///             ("db2.postgres.cluster.local".into(), 5432),
///             ("db3.postgres.cluster.local".into(), 5432),
///             ("db4.postgres.cluster.local".into(), 5432),
///         ],
///         username: "my_username".into(),
///         password: "my password".into(),
///     })
///     .await?;
///
/// let conn = pool.acquire().await?;
///
/// # Ok(())
/// # }
/// ```
pub trait PoolConnector<DB: Database>: Send + Sync + 'static {
    /// Open a connection for the pool.
    ///
    /// Any setup that must be done on the connection should be performed before it is returned.
    ///
    /// If this method returns an error that is known to be retryable, it is called again
    /// in an exponential backoff loop. Retryable errors include, but are not limited to:
    ///
    /// * [`io::ErrorKind::ConnectionRefused`]
    /// * Database errors for which
    ///   [`is_retryable_connect_error`][crate::error::DatabaseError::is_retryable_connect_error]
    ///   returns `true`.
    /// * [`Error::PoolConnector`] with `retryable: true`.
    ///   This error kind is not returned internally and is designed to allow this method to return
    ///   arbitrary error types not otherwise supported.
    ///
    /// Manual implementations of this method may also use the signature:
    /// ```rust,ignore
    /// async fn connect(
    ///     &self,
    ///     meta: PoolConnectMetadata
    /// ) -> sqlx::Result<{PgConnection, MySqlConnection, SqliteConnection, etc.}>
    /// ```
    ///
    /// Note: the returned future must be `Send`.
    fn connect(
        &self,
        meta: PoolConnectMetadata,
    ) -> impl Future<Output = crate::Result<DB::Connection>> + Send + '_;
}

impl<DB, F, Fut> PoolConnector<DB> for F
where
    DB: Database,
    F: Fn(PoolConnectMetadata) -> Fut + Send + Sync + 'static,
    Fut: Future<Output = crate::Result<DB::Connection>> + Send + 'static,
{
    fn connect(
        &self,
        meta: PoolConnectMetadata,
    ) -> impl Future<Output = crate::Result<DB::Connection>> + Send + '_ {
        self(meta)
    }
}

pub(crate) struct DefaultConnector<DB: Database>(
    pub <<DB as Database>::Connection as Connection>::Options,
);

impl<DB: Database> PoolConnector<DB> for DefaultConnector<DB> {
    fn connect(
        &self,
        _meta: PoolConnectMetadata,
    ) -> impl Future<Output = crate::Result<DB::Connection>> + Send + '_ {
        self.0.connect()
    }
}

/// Metadata passed to [`PoolConnector::connect()`] for every connection attempt.
#[derive(Debug)]
#[non_exhaustive]
pub struct PoolConnectMetadata {
    /// The instant at which the current connection task was started, including all attempts.
    ///
    /// May be used for reporting purposes, or to implement a custom backoff.
    pub start: Instant,
    /// The number of attempts that have occurred so far.
    pub num_attempts: usize,
    /// The current size of the pool.
    pub pool_size: usize,
    /// The ID of the connection, unique for the pool.
    pub connection_id: ConnectionId,
}

pub struct DynConnector<DB: Database> {
    // We want to spawn the connection attempt as a task anyway
    connect: Box<
        dyn Fn(ConnectionId, ConnectPermit<DB>) -> JoinHandle<crate::Result<PoolConnection<DB>>>
            + Send
            + Sync
            + 'static,
    >,
}

impl<DB: Database> DynConnector<DB> {
    pub fn new(connector: impl PoolConnector<DB>) -> Self {
        let connector = Arc::new(connector);

        Self {
            connect: Box::new(move |id, permit| {
                crate::rt::spawn(connect_with_backoff(id, permit, connector.clone()))
            }),
        }
    }

    pub fn connect(
        &self,
        id: ConnectionId,
        permit: ConnectPermit<DB>,
    ) -> JoinHandle<crate::Result<PoolConnection<DB>>> {
        (self.connect)(id, permit)
    }
}

pub struct ConnectionCounter {
    count: AtomicUsize,
    next_id: AtomicUsize,
    connect_available: Event,
}

/// An opaque connection ID, unique for every connection attempt with the same pool.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct ConnectionId(usize);

impl ConnectionCounter {
    pub fn new() -> Self {
        Self {
            count: AtomicUsize::new(0),
            next_id: AtomicUsize::new(1),
            connect_available: Event::new(),
        }
    }

    pub fn connections(&self) -> usize {
        self.count.load(Ordering::Acquire)
    }

    pub async fn drain(&self) {
        while self.count.load(Ordering::Acquire) > 0 {
            self.connect_available.listen().await;
        }
    }

    /// Attempt to acquire a permit from both this instance, and the parent pool, if applicable.
    ///
    /// Returns the permit, and the ID of the new connection.
    pub fn try_acquire_permit<DB: Database>(
        &self,
        pool: &Arc<PoolInner<DB>>,
    ) -> Option<(ConnectionId, ConnectPermit<DB>)> {
        debug_assert!(ptr::addr_eq(self, &pool.counter));

        // Don't skip the queue.
        if pool.options.fair && self.connect_available.total_listeners() > 0 {
            return None;
        }

        let prev_size = self
            .count
            .fetch_update(Ordering::Release, Ordering::Acquire, |connections| {
                (connections < pool.options.max_connections).then_some(connections + 1)
            })
            .ok()?;

        let size = prev_size + 1;

        tracing::trace!(target: "sqlx::pool::connect", size, "increased size");

        Some((
            ConnectionId(self.next_id.fetch_add(1, Ordering::SeqCst)),
            ConnectPermit {
                pool: Some(Arc::clone(pool)),
            },
        ))
    }

    /// Attempt to acquire a permit from both this instance, and the parent pool, if applicable.
    ///
    /// Returns the permit, and the current size of the pool.
    pub async fn acquire_permit<DB: Database>(
        &self,
        pool: &Arc<PoolInner<DB>>,
    ) -> (ConnectionId, ConnectPermit<DB>) {
        // Check that `self` can increase size first before we check the parent.
        let acquired = self.acquire_permit_self(pool).await;

        if let Some(parent) = pool.parent() {
            let (_, permit) = parent.0.counter.acquire_permit_self(&parent.0).await;

            // consume the parent permit
            permit.consume();
        }

        acquired
    }

    // Separate method because `async fn`s cannot be recursive.
    /// Attempt to acquire a [`ConnectPermit`] from this instance and this instance only.
    async fn acquire_permit_self<DB: Database>(
        &self,
        pool: &Arc<PoolInner<DB>>,
    ) -> (ConnectionId, ConnectPermit<DB>) {
        for attempt in 1usize.. {
            if let Some(acquired) = self.try_acquire_permit(pool) {
                return acquired;
            }

            self.connect_available.listen().await;

            if attempt == 2 {
                tracing::warn!(
                    "unable to acquire a connect permit after sleeping; this may indicate a bug"
                );
            }
        }

        panic!("BUG: was never able to acquire a connection despite waking many times")
    }

    pub fn release_permit<DB: Database>(&self, pool: &PoolInner<DB>) {
        debug_assert!(ptr::addr_eq(self, &pool.counter));

        self.count.fetch_sub(1, Ordering::Release);
        self.connect_available.notify(1usize);

        if let Some(parent) = &pool.options.parent_pool {
            parent.0.counter.release_permit(&parent.0);
        }
    }
}

pub struct ConnectPermit<DB: Database> {
    pool: Option<Arc<PoolInner<DB>>>,
}

impl<DB: Database> ConnectPermit<DB> {
    pub fn float_existing(pool: Arc<PoolInner<DB>>) -> Self {
        Self { pool: Some(pool) }
    }

    pub fn pool(&self) -> &Arc<PoolInner<DB>> {
        self.pool.as_ref().unwrap()
    }

    pub fn consume(mut self) {
        self.pool = None;
    }
}

impl<DB: Database> Drop for ConnectPermit<DB> {
    fn drop(&mut self) {
        if let Some(pool) = self.pool.take() {
            pool.counter.release_permit(&pool);
        }
    }
}

impl Display for ConnectionId {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        Display::fmt(&self.0, f)
    }
}

#[tracing::instrument(
    target = "sqlx::pool::connect", 
    skip_all,
    fields(%connection_id),
    err
)]
async fn connect_with_backoff<DB: Database>(
    connection_id: ConnectionId,
    permit: ConnectPermit<DB>,
    connector: Arc<impl PoolConnector<DB>>,
) -> crate::Result<PoolConnection<DB>> {
    if permit.pool().is_closed() {
        return Err(Error::PoolClosed);
    }

    let mut ease_off = EaseOff::start_timeout(permit.pool().options.connect_timeout);

    for attempt in 1usize.. {
        let meta = PoolConnectMetadata {
            start: ease_off.started_at(),
            num_attempts: attempt,
            pool_size: permit.pool().size(),
            connection_id,
        };

        let conn = ease_off
            .try_async(connector.connect(meta))
            .await
            .or_retry_if(|e| can_retry_error(e.inner()))?;

        if let Some(conn) = conn {
            return Ok(Floating::new_live(conn, connection_id, permit).reattach());
        }
    }

    Err(Error::PoolTimedOut)
}

fn can_retry_error(e: &Error) -> bool {
    match e {
        Error::Io(e) if e.kind() == io::ErrorKind::ConnectionRefused => true,
        Error::Database(e) => e.is_retryable_connect_error(),
        _ => false,
    }
}
