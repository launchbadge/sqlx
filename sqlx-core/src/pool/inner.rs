use super::connection::ConnectionInner;
use crate::database::Database;
use crate::error::Error;
use crate::pool::{connection, CloseEvent, Pool, PoolConnection, PoolConnector, PoolOptions};

use std::cmp;
use std::future::Future;
use std::pin::{pin, Pin};
use std::rc::Weak;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::task::{ready, Poll};

use crate::connection::Connection;
use crate::logger::private_level_filter_to_trace_level;
use crate::pool::connect::{
    ConnectPermit, ConnectTask, ConnectTaskShared, ConnectionCounter, ConnectionId, DynConnector,
};
use crate::pool::shard::{ConnectedSlot, DisconnectedSlot, Sharded};
use crate::rt::JoinHandle;
use crate::{private_tracing_dynamic_event, rt};
use either::Either;
use futures_core::FusedFuture;
use futures_util::future::{self, OptionFuture};
use futures_util::{stream, FutureExt, TryStreamExt};
use std::time::{Duration, Instant};
use tracing::Level;

const GRACEFUL_CLOSE_TIMEOUT: Duration = Duration::from_secs(5);

pub(crate) struct PoolInner<DB: Database> {
    pub(super) connector: DynConnector<DB>,
    pub(super) counter: ConnectionCounter,
    pub(super) sharded: Sharded<ConnectionInner<DB>>,
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
        let pool = Arc::<Self>::new_cyclic(|pool_weak| {
            let pool_weak = pool_weak.clone();

            let reconnect = move |slot| {
                let Some(pool) = pool_weak.upgrade() else {
                    // Prevent an infinite loop on pool drop.
                    DisconnectedSlot::leak(slot);
                    return;
                };

                pool.connector.connect(
                    Pool(pool.clone()),
                    ConnectionId::next(),
                    slot,
                    ConnectTaskShared::new_arc(),
                );
            };

            Self {
                connector: DynConnector::new(connector),
                counter: ConnectionCounter::new(),
                sharded: Sharded::new(
                    options.max_connections,
                    options.shards,
                    options.min_connections,
                    reconnect,
                ),
                is_closed: AtomicBool::new(false),
                on_closed: event_listener::Event::new(),
                acquire_time_level: private_level_filter_to_trace_level(options.acquire_time_level),
                acquire_slow_level: private_level_filter_to_trace_level(options.acquire_slow_level),
                options,
            }
        });

        spawn_maintenance_tasks(&pool);

        pool
    }

    pub(super) fn size(&self) -> usize {
        self.sharded.count_connected()
    }

    pub(super) fn num_idle(&self) -> usize {
        self.sharded.count_unlocked(true)
    }

    pub(super) fn is_closed(&self) -> bool {
        self.is_closed.load(Ordering::Acquire)
    }

    fn mark_closed(&self) {
        self.is_closed.store(true, Ordering::Release);
        self.on_closed.notify(usize::MAX);
    }

    pub(super) fn close(self: &Arc<Self>) -> impl Future<Output = ()> + '_ {
        self.mark_closed();

        // Keep clearing the idle queue as connections are released until the count reaches zero.
        self.sharded.drain(|slot| async move {
            let (conn, slot) = ConnectedSlot::take(slot);

            let _ = rt::timeout(GRACEFUL_CLOSE_TIMEOUT, conn.raw.close()).await;

            slot
        })
    }

    pub(crate) fn close_event(&self) -> CloseEvent {
        CloseEvent {
            listener: (!self.is_closed()).then(|| self.on_closed.listen()),
        }
    }

    pub(super) fn parent(&self) -> Option<&Pool<DB>> {
        self.options.parent_pool.as_ref()
    }

    #[inline]
    pub(super) fn try_acquire(self: &Arc<Self>) -> Option<ConnectedSlot<ConnectionInner<DB>>> {
        if self.is_closed() {
            return None;
        }

        self.sharded.try_acquire_connected()
    }

    pub(super) async fn acquire(self: &Arc<Self>) -> Result<PoolConnection<DB>, Error> {
        if self.is_closed() {
            return Err(Error::PoolClosed);
        }

        let acquire_started_at = Instant::now();

        let mut close_event = pin!(self.close_event());
        let mut deadline = pin!(rt::sleep(self.options.acquire_timeout));

        let connect_shared = ConnectTaskShared::new_arc();

        let mut acquire_connected = pin!(self.acquire_connected().fuse());

        let mut acquire_disconnected = pin!(self.sharded.acquire_disconnected().fuse());

        let mut connect = future::Fuse::terminated();

        let acquired = std::future::poll_fn(|cx| loop {
            if let Poll::Ready(()) = close_event.as_mut().poll(cx) {
                return Poll::Ready(Err(Error::PoolClosed));
            }

            if let Poll::Ready(res) = acquire_connected.as_mut().poll(cx) {
                match res {
                    Ok(conn) => {
                        return Poll::Ready(Ok(conn));
                    }
                    Err(slot) => {
                        if connect.is_terminated() {
                            connect = self
                                .connector
                                .connect(
                                    Pool(self.clone()),
                                    ConnectionId::next(),
                                    slot,
                                    connect_shared.clone(),
                                )
                                .fuse();
                        }

                        // Try to acquire another connected connection.
                        acquire_connected.set(self.acquire_connected().fuse());
                        continue;
                    }
                }
            }

            if let Poll::Ready(slot) = acquire_disconnected.as_mut().poll(cx) {
                if connect.is_terminated() {
                    connect = self
                        .connector
                        .connect(
                            Pool(self.clone()),
                            ConnectionId::next(),
                            slot,
                            connect_shared.clone(),
                        )
                        .fuse();
                }
            }

            if let Poll::Ready(res) = Pin::new(&mut connect).poll(cx) {
                return Poll::Ready(res);
            }

            if let Poll::Ready(()) = deadline.as_mut().poll(cx) {
                return Poll::Ready(Err(Error::PoolTimedOut {
                    last_connect_error: connect_shared.take_error().map(Box::new),
                }));
            }

            return Poll::Pending;
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
                acquired_after_secs = acquired_after.as_secs_f64(),
                slow_acquire_threshold_secs = self.options.acquire_slow_threshold.as_secs_f64(),
                "acquired connection, but time to acquire exceeded slow threshold"
            );
        } else if let Some(level) = self.acquire_time_level {
            private_tracing_dynamic_event!(
                target: "sqlx::pool::acquire",
                level,
                acquired_after_secs = acquired_after.as_secs_f64(),
                "acquired connection"
            );
        }

        Ok(acquired)
    }

    async fn acquire_connected(
        self: &Arc<Self>,
    ) -> Result<PoolConnection<DB>, DisconnectedSlot<ConnectionInner<DB>>> {
        let connected = self.sharded.acquire_connected().await;

        tracing::debug!(
            target: "sqlx::pool",
            connection_id=%connected.id,
            "acquired idle connection"
        );

        match finish_acquire(self, connected) {
            Either::Left(task) => task.await,
            Either::Right(conn) => Ok(conn),
        }
    }

    pub(crate) async fn try_min_connections(
        self: &Arc<Self>,
        deadline: Option<Instant>,
    ) -> Result<(), Error> {
        let shared = ConnectTaskShared::new_arc();

        let connect_min_connections =
            future::try_join_all(self.sharded.iter_min_connections().map(|slot| {
                self.connector.connect(
                    Pool(self.clone()),
                    ConnectionId::next(),
                    slot,
                    shared.clone(),
                )
            }));

        let mut conns = if let Some(deadline) = deadline {
            match rt::timeout_at(deadline, connect_min_connections).await {
                Ok(Ok(conns)) => conns,
                Err(_) | Ok(Err(Error::PoolTimedOut { .. })) => {
                    return Err(Error::PoolTimedOut {
                        last_connect_error: shared.take_error().map(Box::new),
                    });
                }
                Ok(Err(e)) => return Err(e),
            }
        } else {
            connect_min_connections.await?
        };

        for mut conn in conns {
            // Bypass `after_release`
            drop(conn.return_to_pool());
        }

        Ok(())
    }
}

impl<DB: Database> Drop for PoolInner<DB> {
    fn drop(&mut self) {
        self.mark_closed();
    }
}

/// Returns `true` if the connection has exceeded `options.max_lifetime` if set, `false` otherwise.
pub(super) fn is_beyond_max_lifetime<DB: Database>(
    live: &ConnectionInner<DB>,
    options: &PoolOptions<DB>,
) -> bool {
    options
        .max_lifetime
        .is_some_and(|max| live.created_at.elapsed() > max)
}

/// Returns `true` if the connection has exceeded `options.idle_timeout` if set, `false` otherwise.
fn is_beyond_idle_timeout<DB: Database>(
    idle: &ConnectionInner<DB>,
    options: &PoolOptions<DB>,
) -> bool {
    options
        .idle_timeout
        .is_some_and(|timeout| idle.last_released_at.elapsed() > timeout)
}

/// Execute `test_before_acquire` and/or `before_acquire` in a background task, if applicable.
///
/// Otherwise, immediately returns the connection.
fn finish_acquire<DB: Database>(
    pool: &Arc<PoolInner<DB>>,
    mut conn: ConnectedSlot<ConnectionInner<DB>>,
) -> Either<
    JoinHandle<Result<PoolConnection<DB>, DisconnectedSlot<ConnectionInner<DB>>>>,
    PoolConnection<DB>,
> {
    if pool.options.test_before_acquire || pool.options.before_acquire.is_some() {
        let pool = pool.clone();

        // Spawn a task so the call may complete even if `acquire()` is cancelled.
        return Either::Left(rt::spawn(async move {
            // Check that the connection is still live
            if let Err(error) = conn.raw.ping().await {
                // an error here means the other end has hung up or we lost connectivity
                // either way we're fine to just discard the connection
                // the error itself here isn't necessarily unexpected so WARN is too strong
                tracing::info!(%error, connection_id=%conn.id, "ping on idle connection returned error");

                // connection is broken so don't try to close nicely
                let (_res, slot) = connection::close_hard(conn).await;
                return Err(slot);
            }

            if let Some(test) = &pool.options.before_acquire {
                let meta = conn.idle_metadata();
                match test(&mut conn.raw, meta).await {
                    Ok(false) => {
                        // connection was rejected by user-defined hook, close nicely
                        let (_res, slot) = connection::close(conn).await;
                        return Err(slot);
                    }

                    Err(error) => {
                        tracing::warn!(%error, "error from `before_acquire`");

                        // connection is broken so don't try to close nicely
                        let (_res, slot) = connection::close_hard(conn).await;
                        return Err(slot);
                    }

                    Ok(true) => {}
                }
            }

            Ok(PoolConnection::new(conn, pool))
        }));
    }

    // No checks are configured, return immediately.
    Either::Right(PoolConnection::new(conn, pool.clone()))
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
                rt::spawn(async move {
                    if let Some(pool) = pool_weak.upgrade() {
                        if let Err(error) = pool.try_min_connections(None).await {
                            tracing::error!(
                                target: "sqlx::pool",
                                ?error,
                                "error maintaining min_connections"
                            );
                        }
                    }
                });
            }

            return;
        }
    };

    // Immediately cancel this task if the pool is closed.
    let mut close_event = pool.close_event();

    rt::spawn(async move {
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
                    // open a new one immediately.
                    for conn in pool.sharded.iter_idle() {
                        if is_beyond_idle_timeout(&conn, &pool.options)
                            || is_beyond_max_lifetime(&conn, &pool.options)
                        {
                            // Dropping the slot will check if the connection needs to be
                            // re-made.
                            let _ = connection::close(conn).await;
                        }
                    }

                    // Don't hold a reference to the pool while sleeping.
                    drop(pool);

                    rt::sleep_until(next_run).await;
                }
            })
            .await;
    });
}
