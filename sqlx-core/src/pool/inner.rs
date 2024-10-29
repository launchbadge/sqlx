use super::connection::{Floating, Idle, Live};
use crate::database::Database;
use crate::error::Error;
use crate::pool::{CloseEvent, Pool, PoolConnection, PoolConnector, PoolOptions};

use std::cmp;
use std::future::Future;
use std::pin::pin;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::task::ready;

use crate::logger::private_level_filter_to_trace_level;
use crate::pool::connect::{ConnectPermit, ConnectionCounter, DynConnector};
use crate::pool::idle::IdleQueue;
use crate::private_tracing_dynamic_event;
use futures_util::future::{self, OptionFuture};
use futures_util::FutureExt;
use std::time::{Duration, Instant};
use tracing::Level;

pub(crate) struct PoolInner<DB: Database> {
    pub(super) connector: DynConnector<DB>,
    pub(super) counter: ConnectionCounter,
    pub(super) idle: IdleQueue<DB>,
    is_closed: AtomicBool,
    pub(super) on_closed: event_listener::Event,
    pub(super) options: PoolOptions<DB>,
    pub(crate) acquire_time_level: Option<Level>,
    pub(crate) acquire_slow_level: Option<Level>,
}

impl<DB: Database> PoolInner<DB> {
    pub(super) fn new_arc(
        options: PoolOptions<DB>,
        connector: impl PoolConnector<DB>,
    ) -> Arc<Self> {
        let pool = Self {
            connector: DynConnector::new(connector),
            counter: ConnectionCounter::new(),
            idle: IdleQueue::new(options.fair, options.max_connections),
            is_closed: AtomicBool::new(false),
            on_closed: event_listener::Event::new(),
            acquire_time_level: private_level_filter_to_trace_level(options.acquire_time_level),
            acquire_slow_level: private_level_filter_to_trace_level(options.acquire_slow_level),
            options,
        };

        let pool = Arc::new(pool);

        spawn_maintenance_tasks(&pool);

        pool
    }

    pub(super) fn size(&self) -> usize {
        self.counter.connections()
    }

    pub(super) fn num_idle(&self) -> usize {
        self.idle.len()
    }

    pub(super) fn is_closed(&self) -> bool {
        self.is_closed.load(Ordering::Acquire)
    }

    fn mark_closed(&self) {
        self.is_closed.store(true, Ordering::Release);
        self.on_closed.notify(usize::MAX);
    }

    pub(super) fn close<'a>(self: &'a Arc<Self>) -> impl Future<Output = ()> + 'a {
        self.mark_closed();

        async move {
            while let Some(idle) = self.idle.try_acquire(self) {
                idle.close().await;
            }

            self.counter.drain().await;
        }
    }

    pub(crate) fn close_event(&self) -> CloseEvent {
        CloseEvent {
            listener: (!self.is_closed()).then(|| self.on_closed.listen()),
        }
    }

    fn parent(&self) -> Option<&Pool<DB>> {
        self.options.parent_pool.as_ref()
    }

    #[inline]
    pub(super) fn try_acquire(self: &Arc<Self>) -> Option<Floating<DB, Idle<DB>>> {
        if self.is_closed() {
            return None;
        }

        self.idle.try_acquire(self)
    }

    pub(super) fn release(&self, floating: Floating<DB, Live<DB>>) {
        // `options.after_release` and other checks are in `PoolConnection::return_to_pool()`.
        self.idle.release(floating);
    }

    pub(super) async fn acquire(self: &Arc<Self>) -> Result<PoolConnection<DB>, Error> {
        if self.is_closed() {
            return Err(Error::PoolClosed);
        }

        let acquire_started_at = Instant::now();

        let mut close_event = pin!(self.close_event());
        let mut deadline = pin!(crate::rt::sleep(self.options.acquire_timeout));
        let mut acquire_idle = pin!(self.idle.acquire(self).fuse());
        let mut check_idle = pin!(OptionFuture::from(None));
        let mut acquire_connect_permit = pin!(OptionFuture::from(Some(
            self.counter.acquire_permit(self).fuse()
        )));
        let mut connect = OptionFuture::from(None);

        // The internal state machine of `acquire()`.
        //
        // * The initial state is racing to acquire either an idle connection or a new `ConnectPermit`.
        // * If we acquire a `ConnectPermit`, we begin the connection loop (with backoff)
        //   as implemented by `DynConnector`.
        // * If we acquire an idle connection, we then start polling `check_idle_conn()`.
        let acquired = future::poll_fn(|cx| {
            use std::task::Poll::*;

            // First check if the pool is already closed,
            // or register for a wakeup if it gets closed.
            if let Ready(()) = close_event.poll_unpin(cx) {
                return Ready(Err(Error::PoolClosed));
            }

            // Then check if our deadline has elapsed, or schedule a wakeup for when that happens.
            if let Ready(()) = deadline.poll_unpin(cx) {
                return Ready(Err(Error::PoolTimedOut));
            }

            // Attempt to acquire a connection from the idle queue.
            if let Ready(idle) = acquire_idle.poll_unpin(cx) {
                check_idle.set(Some(check_idle_conn(idle, &self.options)).into());
            }

            // If we acquired an idle connection, run any checks that need to be done.
            //
            // Includes `test_on_acquire` and the `before_acquire` callback, if set.
            //
            // We don't want to race this step if it's already running because canceling it
            // will result in the potentially unnecessary closure of a connection.
            //
            // Instead, we just wait and see what happens. If we already started connecting,
            // that'll happen concurrently.
            match ready!(check_idle.poll_unpin(cx)) {
                // The `.reattach()` call errors with "type annotations needed" if not qualified.
                Some(Ok(live)) => return Ready(Ok(Floating::reattach(live))),
                Some(Err(permit)) => {
                    // We don't strictly need to poll `connect` here; all we really want to do
                    // is to check if it is `None`. But since currently there's no getter for that,
                    // it doesn't really hurt to just poll it here.
                    match connect.poll_unpin(cx) {
                        Ready(None) => {
                            // If we're not already attempting to connect,
                            // take the permit returned from closing the connection and
                            // attempt to open a new one.
                            connect = Some(self.connector.connect(permit, self.size())).into();
                        }
                        // `permit` is dropped in these branches, allowing another task to use it
                        Ready(Some(res)) => return Ready(res),
                        Pending => (),
                    }

                    // Attempt to acquire another idle connection concurrently to opening a new one.
                    acquire_idle.set(self.idle.acquire(self).fuse());
                    // Annoyingly, `OptionFuture` doesn't fuse to `None` on its own
                    check_idle.set(None.into());
                }
                None => (),
            }

            if let Ready(Some((size, permit))) = acquire_connect_permit.poll_unpin(cx) {
                connect = Some(self.connector.connect(permit, size)).into();
            }

            if let Ready(Some(res)) = connect.poll_unpin(cx) {
                // RFC: suppress errors here?
                return Ready(res);
            }

            Pending
        })
        .await?;

        let acquired_after = acquire_started_at.elapsed();

        let acquire_slow_level = self
            .acquire_slow_level
            .filter(|_| acquired_after > self.options.acquire_slow_threshold);

        if let Some(level) = acquire_slow_level {
            private_tracing_dynamic_event!(
                target: "sqlx::pool::acquire",
                level,
                aquired_after_secs = acquired_after.as_secs_f64(),
                slow_acquire_threshold_secs = self.options.acquire_slow_threshold.as_secs_f64(),
                "acquired connection, but time to acquire exceeded slow threshold"
            );
        } else if let Some(level) = self.acquire_time_level {
            private_tracing_dynamic_event!(
                target: "sqlx::pool::acquire",
                level,
                aquired_after_secs = acquired_after.as_secs_f64(),
                "acquired connection"
            );
        }

        Ok(acquired)
    }

    /// Try to maintain `min_connections`, returning any errors (including `PoolTimedOut`).
    pub async fn try_min_connections(self: &Arc<Self>, deadline: Instant) -> Result<(), Error> {
        crate::rt::timeout_at(deadline, async {
            while self.size() < self.options.min_connections {
                // Don't wait for a connect permit.
                //
                // If no extra permits are available then we shouldn't be trying to spin up
                // connections anyway.
                let Some((size, permit)) = self.counter.acquire_permit(self).now_or_never() else {
                    return Ok(());
                };

                let conn = self.connector.connect(permit, size).await?;

                // We skip `after_release` since the connection was never provided to user code
                // besides inside `PollConnector::connect()`, if they override it.
                self.release(conn.into_floating());
            }

            Ok(())
        })
        .await
        .unwrap_or_else(|_| Err(Error::PoolTimedOut))
    }

    /// Attempt to maintain `min_connections`, logging if unable.
    pub async fn min_connections_maintenance(self: &Arc<Self>, deadline: Option<Instant>) {
        let deadline = deadline.unwrap_or_else(|| {
            // Arbitrary default deadline if the caller doesn't care.
            Instant::now() + Duration::from_secs(300)
        });

        match self.try_min_connections(deadline).await {
            Ok(()) => (),
            Err(Error::PoolClosed) => (),
            Err(Error::PoolTimedOut) => {
                tracing::debug!("unable to complete `min_connections` maintenance before deadline")
            }
            Err(error) => tracing::debug!(%error, "error while maintaining min_connections"),
        }
    }
}

impl<DB: Database> Drop for PoolInner<DB> {
    fn drop(&mut self) {
        self.mark_closed();
        self.idle.drain(self);
    }
}

/// Returns `true` if the connection has exceeded `options.max_lifetime` if set, `false` otherwise.
pub(super) fn is_beyond_max_lifetime<DB: Database>(
    live: &Live<DB>,
    options: &PoolOptions<DB>,
) -> bool {
    options
        .max_lifetime
        .map_or(false, |max| live.created_at.elapsed() > max)
}

/// Returns `true` if the connection has exceeded `options.idle_timeout` if set, `false` otherwise.
fn is_beyond_idle_timeout<DB: Database>(idle: &Idle<DB>, options: &PoolOptions<DB>) -> bool {
    options
        .idle_timeout
        .map_or(false, |timeout| idle.idle_since.elapsed() > timeout)
}

async fn check_idle_conn<DB: Database>(
    mut conn: Floating<DB, Idle<DB>>,
    options: &PoolOptions<DB>,
) -> Result<Floating<DB, Live<DB>>, ConnectPermit<DB>> {
    if options.test_before_acquire {
        // Check that the connection is still live
        if let Err(error) = conn.ping().await {
            // an error here means the other end has hung up or we lost connectivity
            // either way we're fine to just discard the connection
            // the error itself here isn't necessarily unexpected so WARN is too strong
            tracing::info!(%error, "ping on idle connection returned error");
            // connection is broken so don't try to close nicely
            return Err(conn.close_hard().await);
        }
    }

    if let Some(test) = &options.before_acquire {
        let meta = conn.metadata();
        match test(&mut conn.live.raw, meta).await {
            Ok(false) => {
                // connection was rejected by user-defined hook, close nicely
                return Err(conn.close().await);
            }

            Err(error) => {
                tracing::warn!(%error, "error from `before_acquire`");
                // connection is broken so don't try to close nicely
                return Err(conn.close_hard().await);
            }

            Ok(true) => {}
        }
    }

    // No need to re-connect; connection is alive or we don't care
    Ok(conn.into_live())
}

fn spawn_maintenance_tasks<DB: Database>(pool: &Arc<PoolInner<DB>>) {
    // NOTE: use `pool_weak` for the maintenance tasks
    // so they don't keep `PoolInner` from being dropped.
    let pool_weak = Arc::downgrade(pool);

    let period = match (pool.options.max_lifetime, pool.options.idle_timeout) {
        (Some(it), None) | (None, Some(it)) => it,

        (Some(a), Some(b)) => cmp::min(a, b),

        (None, None) => {
            if pool.options.min_connections > 0 {
                crate::rt::spawn(async move {
                    if let Some(pool) = pool_weak.upgrade() {
                        pool.min_connections_maintenance(None).await;
                    }
                });
            }

            return;
        }
    };

    // Immediately cancel this task if the pool is closed.
    let mut close_event = pool.close_event();

    crate::rt::spawn(async move {
        let _ = close_event
            .do_until(async {
                // If the last handle to the pool was dropped while we were sleeping
                while let Some(pool) = pool_weak.upgrade() {
                    if pool.is_closed() {
                        return;
                    }

                    let next_run = Instant::now() + period;

                    // Go over all idle connections, check for idleness and lifetime,
                    // and if we have fewer than min_connections after reaping a connection,
                    // open a new one immediately. Note that other connections may be popped from
                    // the queue in the meantime - that's fine, there is no harm in checking more
                    for _ in 0..pool.num_idle() {
                        if let Some(conn) = pool.try_acquire() {
                            if is_beyond_idle_timeout(&conn, &pool.options)
                                || is_beyond_max_lifetime(&conn, &pool.options)
                            {
                                let _ = conn.close().await;
                                pool.min_connections_maintenance(Some(next_run)).await;
                            } else {
                                pool.release(conn.into_live());
                            }
                        }
                    }

                    // Don't hold a reference to the pool while sleeping.
                    drop(pool);

                    if let Some(duration) = next_run.checked_duration_since(Instant::now()) {
                        // `async-std` doesn't have a `sleep_until()`
                        crate::rt::sleep(duration).await;
                    } else {
                        // `next_run` is in the past, just yield.
                        crate::rt::yield_now().await;
                    }
                }
            })
            .await;
    });
}
