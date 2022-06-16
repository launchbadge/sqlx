use std::cmp;
use std::fmt::{self, Formatter};
use std::ops::Index;
use std::sync::atomic::{self, AtomicU64, Ordering};
use std::sync::Arc;
use std::time::Duration;

use enum_map::EnumMap;

// Saves a bunch of redundant links in docs.
// Just `#[cfg(doc)]` doesn't work for some reason.
use crate::pool::metrics::{AcquirePhase, PoolMetricsCollector};
#[cfg_attr(not(doc), allow(unused_imports))]
use crate::pool::{Pool, PoolOptions};

/// A simple but hopefully useful metrics collector for [`Pool`].
///
/// See [`SimplePoolMetricsSnapshot`] for the metrics collected by this implementation.
///
/// # Example
/// This example is written for PostgreSQL and Tokio but can trivially be adapted
/// to other databases and/or async-std.
///
/// ```no_run
/// # #[cfg(feature = "postgres")]
/// # async fn f() -> Result<(), Box<dyn std::error::Error>> {
/// use sqlx::Executor;
/// use sqlx::postgres::PgPoolOptions;
/// use sqlx::pool::metrics::SimplePoolMetrics;
///
/// let metrics = SimplePoolMetrics::new();
///
/// let pool = PgPoolOptions::new()
///     .metrics_collector(metrics.collector())
///     .connect("postgres:// …")
///     .await?;
///
/// tokio::spawn(async move {
///     // Post metrics every minute.
///     tokio::time::sleep(tokio::time::Duration::from_secs(60)).await;
///
///     // Warning: very verbose!
///     println!("current pool metrics: {:#?}", metrics.snapshot());
/// });
///
/// // use `pool`...
///
/// # Ok(())
/// # }
/// ```
#[derive(Clone)]
pub struct SimplePoolMetrics {
    inner: Arc<SimpleMetricsInner>,
}

/// A snapshot of metrics returned by [`SimplePoolMetrics::snapshot()`].
///
/// If the `json` feature is enabled, this type implements `serde::Serialize`.
#[derive(Debug, Clone)]
#[cfg_attr(feature = "json", derive(serde::Serialize))]
#[non_exhaustive]
pub struct SimplePoolMetricsSnapshot {
    /// Total number of calls to [`Pool::acquire()`] when the snapshot was taken.
    pub acquire_calls: u64,

    /// Statistics for the time [`Pool::acquire()`] spent in [`AcquirePhase::Waiting`].
    pub permit_wait_time: SimpleTimingStats,

    /// Statistics for the time [`Pool::acquire()`] takes to acquire a connection.
    pub acquire_time: SimpleTimingStats,

    /// Total number of times [`Pool::acquire()`] timed out.
    pub acquire_timeouts: u64,

    /// Total number of times [`Pool::acquire()`] timed out aggregated per [`AcquirePhase`] in which
    /// the timeout occurred.
    ///
    /// The value type can be indexed by `AcquirePhase`.
    ///
    /// ```rust
    /// use sqlx::pool::metrics::{AcquirePhase, SimplePoolMetrics, SimplePoolMetricsSnapshot};
    ///
    /// let metrics: SimplePoolMetrics = SimplePoolMetrics::new();
    ///
    /// // pass `metrics.collector()` to `PoolOptions::metrics_collector()`
    /// // then construct and start using the `Pool`
    ///
    /// // sometime later...
    ///
    /// let snapshot: SimplePoolMetricsSnapshot = metrics.snapshot();
    ///
    /// println!(
    ///     "number of times the pool timed out waiting for a permit = {}",
    ///     snapshot.acquire_timeouts_per_phase[AcquirePhase::Waiting]
    /// );
    /// ```
    pub acquire_timeouts_per_phase: AcquireTimeoutsPerPhase,
}

/// The statistics for an individual [`Pool`] timing metric collected by [`SimplePoolMetrics`].
#[derive(Debug, Clone)]
#[cfg_attr(feature = "json", derive(serde::Serialize))]
#[non_exhaustive]
pub struct SimpleTimingStats {
    /// The total count of samples collected for this metric.
    pub sample_count: u64,

    /// The minimum time for this metric. [`Duration::ZERO`] if no samples were collected.
    pub min: Duration,

    /// The average time for this metric, calculated as an [Exponential Moving Average].
    ///
    /// [`Duration::ZERO`] if no samples were collected.
    ///
    /// The EMA coefficient is set during construction of [`SimplePoolMetrics`].
    /// See [`SimplePoolMetrics::with_ema_coefficient()`] for details.
    ///
    /// [Exponential Moving Average]: https://en.wikipedia.org/wiki/Moving_average#Exponential_moving_average
    pub average: Duration,

    /// The maximum time for this metric. [`Duration::ZERO`] if no samples were collected.
    pub max: Duration,
}

/// Counts of [`Pool::acquire()`] timeouts aggregated per [`AcquirePhase`] in which the timeout occurred.
#[derive(Debug, Clone)]
#[cfg_attr(feature = "json", derive(serde::Serialize))]
pub struct AcquireTimeoutsPerPhase(EnumMap<AcquirePhase, u64>);

#[derive(Default)]
struct SimpleMetricsInner {
    ema_coefficient: f64,
    acquire_calls: AtomicU64,
    permit_wait_time: AtomicTimingStats,
    acquire_time: AtomicTimingStats,
    acquire_timeouts: AtomicU64,
    acquire_timeouts_per_phase: EnumMap<AcquirePhase, AtomicU64>,
}

#[derive(Default)]
struct AtomicTimingStats {
    sample_count: AtomicU64,
    min_nanos: AtomicU64,
    average_nanos: AtomicU64,
    max_nanos: AtomicU64,
}

impl SimplePoolMetrics {
    /// Construct with default settings.
    pub fn new() -> SimplePoolMetrics {
        // Arbitrarily chosen, but should give decent metrics.
        // See the table in the docs below for details.
        Self::with_ema_coefficient(0.01)
    }

    /// Construct with the given coefficient for calculating [Exponential Moving Averages].
    ///
    /// `ema_coefficient` is the factor `α` in the following formula:
    ///
    /// <img
    ///  src="https://wikimedia.org/api/rest_v1/media/math/render/svg/692012ed1c78d38cbbe6ad9935786c80ec7c24de"
    ///  style="filter: invert(100%)"></img>
    ///
    /// Essentially, it determines how much new samples influence the average. A smaller coefficient
    /// produces a more stable but more slowly moving average, a larger coefficient produces
    /// a quickly moving but chaotic average.
    ///
    /// The following table shows how much each sample contributes to the average
    /// for some arbitrary coefficients, where the Nth sample is the latest:
    ///
    // Got kinda nerd sniped calculating this table, tbh.
    // I was trying to demonstrate how quickly each coefficient makes old samples irrelevant.
    /// | α =   | 0.01  | 0.05  | 0.1      | 0.2     | 0.25   | 0.5     |
    /// |-------|-------|-------|----------|---------|--------|---------|
    /// | N     | 1%    | 5%    | 10%      | 20%     | 25%    | 50%     |
    /// | N-1   | 0.99% | 4.75% | 9%       | 16%     | 18.75% | 25%     |
    /// | N-2   | 0.98% | 4.51% | 8.1%     | 12.8%   | 14.06% | 12.5%   |
    /// | N-3   | 0.97% | 4.29% | 7.29%    | 10.24%  | 10.54% | 6.25%   |
    /// | N-4   | 0.96% | 4.07% | 6.56%    | 8.19%   | 7.91%  | 3.125%  |
    /// | ⋮     |       | ⋮     | ⋮        | ⋮       | ⋮      | ⋮       |
    /// | N-10  | 0.90% | 2.99% | 3.4%     | 2.15%   | 1.41%  | 0.049%  |
    /// | ⋮     |       | ⋮     | ⋮        | ⋮       | ⋮      | ⋮       |
    /// | N-20  | 0.82% | 1.79% | 1.22%    | 0.23%   | 0.079% | 4.8 ppb |
    /// | ⋮     |       | ⋮     | ⋮        | ⋮       | ⋮      | ⋮       |
    /// | N-100 | 0.36% | 0.03% | 26.5 ppb | 0.4 ppt | <1 ppt | <1 ppt  |
    ///
    /// For coefficients greater than ~0.19, the N-100th sample contributes less than
    /// one part per trillion to the average.  
    /// Greater than ~0.13, less than one part per billion.  
    /// Greater than ~0.6, less than one part per million.
    ///
    /// ### Panics
    /// If `ema_coefficient` is outside the range `(0, 1)` or is non-normal.
    ///
    /// A coefficient of zero causes the average to never change.
    /// A coefficient of one causes the average to always be equal to the last sample.
    /// In either case, it's no longer an average.
    ///
    /// [Exponential Moving Averages]: https://en.wikipedia.org/wiki/Moving_average#Exponential_moving_average
    pub fn with_ema_coefficient(ema_coefficient: f64) -> Self {
        assert!(ema_coefficient.is_normal());
        assert!(ema_coefficient > 0.0);
        assert!(ema_coefficient < 1.0);

        SimplePoolMetrics {
            inner: Arc::new(SimpleMetricsInner {
                ema_coefficient,
                acquire_calls: AtomicU64::new(0),
                permit_wait_time: AtomicTimingStats::default(),
                acquire_time: AtomicTimingStats::default(),
                acquire_timeouts: AtomicU64::new(0),
                acquire_timeouts_per_phase: EnumMap::default(),
            }),
        }
    }

    /// Get the collector instance to pass to [`PoolOptions::metrics_collector()`].
    pub fn collector(&self) -> Arc<dyn PoolMetricsCollector> {
        self.inner.clone()
    }

    /// Get the current count of calls to [`Pool::acquire()`].
    ///
    /// If you want to inspect multiple statistics at once,
    /// [`.snapshot()`][Self::snapshot] is more efficient.
    pub fn acquire_calls(&self) -> u64 {
        self.inner.acquire_calls.load(Ordering::Acquire)
    }

    /// Get the current statistics for the time [`Pool::acquire()`] spends in [`AcquirePhase::Waiting`].
    ///
    /// If you want to inspect multiple statistics at once,
    /// [`.snapshot()`][Self::snapshot] is more efficient.
    pub fn permit_wait_time(&self) -> SimpleTimingStats {
        atomic::fence(Ordering::Acquire);
        self.inner.permit_wait_time.get()
    }

    /// Get the current statistics for the total time [`Pool::acquire()`] takes to get a connection.
    ///
    /// If you want to inspect multiple statistics at once,
    /// [`.snapshot()`][Self::snapshot] is more efficient.
    pub fn acquire_time(&self) -> SimpleTimingStats {
        atomic::fence(Ordering::Acquire);
        self.inner.acquire_time.get()
    }

    /// Load the current values for all metrics.
    ///
    /// More efficient than calling individual getters.
    pub fn snapshot(&self) -> SimplePoolMetricsSnapshot {
        use Ordering::*;

        atomic::fence(Acquire);

        SimplePoolMetricsSnapshot {
            acquire_calls: self.inner.acquire_calls.load(Relaxed),
            permit_wait_time: self.inner.permit_wait_time.get(),
            acquire_time: self.inner.acquire_time.get(),
            acquire_timeouts: self.inner.acquire_timeouts.load(Relaxed),
            acquire_timeouts_per_phase: AcquireTimeoutsPerPhase(
                self.inner
                    .acquire_timeouts_per_phase
                    .iter()
                    .map(|(phase, count)| (phase, count.load(Relaxed)))
                    .collect(),
            ),
        }
    }
}

/// Debug-prints the current metrics as determined by [`Self::snapshot()`].
impl fmt::Debug for SimplePoolMetrics {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.debug_struct("SimplePoolMetrics")
            .field("current", &self.snapshot())
            .finish()
    }
}

impl PoolMetricsCollector for SimpleMetricsInner {
    fn acquire_called(&self) {
        self.acquire_calls.fetch_add(1, Ordering::AcqRel);
    }

    fn permit_wait_time(&self, duration: Duration) {
        self.permit_wait_time.update(self.ema_coefficient, duration)
    }

    fn acquire_timed_out(&self, phase: AcquirePhase) {
        self.acquire_timeouts.fetch_add(1, Ordering::AcqRel);
        self.acquire_timeouts_per_phase[phase].fetch_add(1, Ordering::AcqRel);
    }

    fn connection_acquired(&self, total_wait: Duration) {
        self.acquire_time.update(self.ema_coefficient, total_wait);
    }
}

impl AtomicTimingStats {
    fn update(&self, ema_coefficient: f64, time_sample: Duration) {
        use Ordering::*;

        // If your application triggers this assert then either an `.elapsed()` call overflowed or
        // you somehow kept it running for ~585 years, so congratulate yourself on a job well done.
        let nanos: u64 = time_sample
            .as_nanos()
            .try_into()
            .expect("BUG: `duration` is too large!");

        // Since this is just collecting some statistics, consistency isn't *too* important.
        // We use relaxed orderings for all internal updates and just emit a single fence to
        // get some semblance of synchronization.
        atomic::fence(Acquire);

        self.sample_count.fetch_add(1, Relaxed);

        let _ = self.min_nanos.fetch_update(Relaxed, Relaxed, |prev| {
            if prev == 0 {
                // If our minimum is exactly zero, then we likely haven't collected any samples yet.
                return Some(nanos);
            }

            Some(cmp::min(prev, nanos))
        });

        let _ = self
            .average_nanos
            .fetch_update(Relaxed, Relaxed, |average| {
                if average == 0 {
                    // If we don't have an average, just use our first sample.
                    return Some(nanos);
                }

                // Exponential Moving Average algorithm
                Some(
                    ((nanos as f64 * ema_coefficient) + (average as f64 * (1.0 - ema_coefficient)))
                        as u64,
                )
            });

        let _ = self
            .max_nanos
            .fetch_update(Relaxed, Relaxed, |prev| Some(cmp::max(prev, nanos)));

        // Suggest that our update be published to main memory.
        atomic::fence(Release);
    }

    /// Assumes an atomic fence is issued first.
    fn get(&self) -> SimpleTimingStats {
        use Ordering::*;

        SimpleTimingStats {
            sample_count: self.sample_count.load(Relaxed),
            min: Duration::from_nanos(self.min_nanos.load(Relaxed)),
            average: Duration::from_nanos(self.average_nanos.load(Relaxed)),
            max: Duration::from_nanos(self.max_nanos.load(Relaxed)),
        }
    }
}

impl Index<AcquirePhase> for AcquireTimeoutsPerPhase {
    type Output = u64;

    fn index(&self, index: AcquirePhase) -> &u64 {
        &self.0[index]
    }
}
