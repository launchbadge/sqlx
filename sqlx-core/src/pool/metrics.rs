//! Pool metric / event instrumentation.

use std::{ops::Deref, sync::Arc, time::Duration};

/// An observer of [`Pool`] metrics.
///
/// ```rust
/// use std::sync::Arc;
/// use sqlx::pool::PoolMetricsObserver;
///
/// #[derive(Default)]
/// struct Observer;
///
/// impl PoolMetricsObserver for Observer {
///     fn permit_wait_time(&self, time: std::time::Duration) {
///         println!(
///             "waited {} milliseconds to get a slot in the connection pool",
///             time.as_millis()
///         );
///     }
/// }
///
/// # #[cfg(feature = "any")]
/// # async fn _example() -> Result<(), sqlx::Error> {
/// # let database_url = "";
/// // Initialise the observer as a dyn PoolMetricsObserver
/// let metrics: Arc<dyn PoolMetricsObserver> = Arc::new(Observer::default());
///
/// // Configure the pool to push metrics to the observer
/// # use sqlx_core::any::AnyPoolOptions;
/// # use sqlx::Executor;
/// let pool = AnyPoolOptions::new()
///     .max_connections(1)
///     .metrics_observer(Arc::clone(&metrics))
///     .connect(&database_url)
///     .await?;
///
/// // Use the pool and see the wait times!
/// pool.execute("SELECT 1;").await?;
/// # Ok(())
/// # }
/// ```
///
/// [`Pool`]: crate::pool::Pool
pub trait PoolMetricsObserver: Send + Sync {
    /// Called with the [`Duration`] spent waiting on a permit for a connection
    /// to be granted from the underlying connection pool, each time a permit
    /// acquisition attempt completes (successfully or not).
    ///
    /// # Blocking
    ///
    /// The [`acquire()`] call blocks while this method is called by the
    /// connection pool. Implementations should aim to return relatively
    /// quickly.
    ///
    /// # Semantics
    ///
    /// This value is incremented once a connection permit is granted, and does
    /// NOT include the time taken to perform any liveness checks on connections
    /// or time taken to establish a connection, if needed.
    ///
    /// If a [connection timeout][1] expires while waiting for a connection from
    /// the pool, the duration of time waiting for the permit is included in
    /// this measurement.
    ///
    /// NOTE: this may report a small wait duration even if connection permits
    /// are immediately available when calling [`acquire()`], as acquiring one
    /// is not instantaneous.
    ///
    /// [1]: crate::pool::PoolOptions::connect_timeout()
    /// [`acquire()`]: crate::pool::Pool::acquire()
    /// [`Pool`]: crate::pool::Pool
    fn permit_wait_time(&self, time: Duration) {
        let _ = time;
    }
}

impl<T> PoolMetricsObserver for Arc<T>
where
    T: PoolMetricsObserver,
{
    fn permit_wait_time(&self, time: Duration) {
        self.deref().permit_wait_time(time)
    }
}

impl PoolMetricsObserver for Option<Arc<dyn PoolMetricsObserver>> {
    fn permit_wait_time(&self, time: Duration) {
        if let Some(v) = self {
            v.permit_wait_time(time)
        }
    }
}
