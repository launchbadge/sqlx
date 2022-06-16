//! Metrics collection utilities for [`Pool`][crate::pool::Pool].
//!
//!

use std::sync::Arc;
use std::time::Duration;

// Saves a bunch of redundant links in docs.
// Just `#[cfg(doc)]` doesn't work for some reason.
#[cfg_attr(not(doc), allow(unused_imports))]
use {
    crate::connection::Connection,
    crate::pool::{Pool, PoolOptions},
};

mod simple;

pub use simple::{
    AcquireTimeoutsPerPhase, SimplePoolMetrics, SimplePoolMetricsSnapshot, SimpleTimingStats,
};

/// Describes a type that can collect metrics from [`Pool`].
///
/// You can set the metrics collector for a `Pool` instance using [`PoolOptions::metrics_collector`].
///
/// For an easy-start implementation, see [`SimplePoolMetrics`].
///
/// All methods on this trait have provided impls so you can override just the ones you care about.
pub trait PoolMetricsCollector: Send + Sync + 'static {
    /// Record when [`Pool::acquire()`] is called.
    fn acquire_called(&self) {}

    /// Record how long a [`Pool::acquire()`] call waited for a semaphore permit.
    ///
    /// This is the first stage of `acquire()` and gives the call the right-of-way to either
    /// pop a connection from the idle queue or open a new one.
    ///
    /// This time is likely to increase as the pool comes under higher and higher load,
    /// and will asymptotically approach the [acquire timeout][PoolOptions::acquire_timeout].
    ///
    /// If `acquire()` times out while waiting for a permit, this method will not be called.  
    /// You will get an <code>acquire_timed_out([AcquirePhase::Waiting])</code> call instead.
    ///
    /// [acquire_timed_out]: Self::acquire_timed_out
    fn permit_wait_time(&self, duration: Duration) {
        drop(duration);
    }

    /// Record when [`Pool::acquire()`] times out as governed by [`PoolOptions::acquire_timeout`].
    ///
    /// `acquire()` has several internal asynchronous operations that it may time out on.  
    /// The given [`AcquirePhase`] tells you which one timed out.
    fn acquire_timed_out(&self, phase: AcquirePhase) {
        drop(phase);
    }

    /// Record when a connection is successfully acquired.
    fn connection_acquired(&self, total_wait: Duration) {
        drop(total_wait);
    }
}

macro_rules! opt_delegate {
    ($receiver:ident.$method:ident $( ( $($arg:expr),*) )?) => {
        if let Some(this) = $receiver {
            this.$method($( $($arg),* )?);
        }
    }
}

#[doc(hidden)]
impl PoolMetricsCollector for Option<Arc<dyn PoolMetricsCollector>> {
    fn acquire_called(&self) {
        opt_delegate!(self.acquire_called());
    }

    fn permit_wait_time(&self, duration: Duration) {
        opt_delegate!(self.permit_wait_time(duration));
    }

    fn acquire_timed_out(&self, phase: AcquirePhase) {
        opt_delegate!(self.acquire_timed_out(phase));
    }

    fn connection_acquired(&self, total_wait: Duration) {
        opt_delegate!(self.connection_acquired(total_wait));
    }
}

/// The phase that [`Pool::acquire()`] was in when it timed out.
///
/// [`Pool::acquire()`] has several internal asynchronous operations, any of which may lead
/// to it timing out. Which phases are executed depends on multiple things:
///
/// * The pool's configuration.
/// * If an idle connection was available or not.
/// * If there is room in the pool for a new connection.
///
/// ### Note: Some Trait impls are Unstable
/// The `enum_map` trait impls are *not* considered part of the stable API.
/// They would not be listed in documentation if it was possible to tell the derive to hide them.
///
/// We reserve the right to update `enum_map` to a non-compatible version if necessary.
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, enum_map::Enum)]
#[cfg_attr(feature = "json", derive(serde::Serialize))]
#[non_exhaustive]
pub enum AcquirePhase {
    /// Initial [`Pool::acquire()`] phase: waiting for a semaphore permit.
    ///
    /// A permit represents the privilege to acquire a connection, either by popping one
    /// from the idle queue or opening a new one.
    Waiting,

    /// `acquire()` found an idle connection. It then calls [`Connection::ping()`] on it.
    ///
    /// Only done if [`PoolOptions::test_before_acquire`] is `true` (enabled by default).
    TestBeforeAcquire,

    /// `acquire()` found an idle connection and the `TestBeforeAcquire` phase succeeded
    /// or was skipped.
    ///
    /// It then invokes the user-defined [`before_acquire`][PoolOptions::before_acquire] callback, if set.
    BeforeAcquireCallback,

    /// `acquire()` found an idle connection but decided to close it.
    ///
    /// This may have happened for any of the following reasons:
    /// * The connection's age exceeded [`PoolOptions::max_lifetime`].
    /// * The `TestBeforeAcquire` phase failed.
    /// * The `BeforeAcquireCallback` errored or rejected the connection.
    /// * A new connection was opened but the `AfterConnectCallback` phase errored.
    ClosingInvalidConnection,

    /// `acquire()` either did not find an idle connection or the connection it got failed
    /// the `TestBeforeAcquire` or `BeforeAcquireCallback` phase and was closed.
    ///
    /// It then attempted to open a new connection.
    Connecting,

    /// `acquire()` successfully opened a new connection.
    ///
    /// It then invokes the user-defined [`after_connect`][PoolOptions::after_connect] callback, if set.
    AfterConnectCallback,

    /// `acquire()` failed to open a new connection or the connection failed the
    /// `AfterConnectCallback` phase.
    ///
    /// It then waits in a backoff loop before attempting to open another connection.
    Backoff,
}
