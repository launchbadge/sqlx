use super::connection::{Floating, Idle, Live};
use crate::connection::ConnectOptions;
use crate::connection::Connection;
use crate::database::Database;
use crate::error::Error;
use crate::pool::{deadline_as_timeout, CloseEvent, Pool, PoolOptions};
use crossbeam_queue::ArrayQueue;

use futures_intrusive::sync::{Semaphore, SemaphoreReleaser};

use std::cmp;
use std::future::Future;
use std::sync::atomic::{AtomicBool, AtomicU32, AtomicUsize, Ordering};
use std::sync::Arc;
use std::task::Poll;

use crate::pool::options::PoolConnectionMetadata;
use futures_util::future::{self};
use futures_util::FutureExt;
use std::time::{Duration, Instant};

pub(crate) struct PoolInner<DB: Database> {
    pub(super) connect_options: <DB::Connection as Connection>::Options,
    pub(super) idle_conns: ArrayQueue<Idle<DB>>,
    pub(super) semaphore: Semaphore,
    pub(super) size: AtomicU32,
    pub(super) num_idle: AtomicUsize,
    is_closed: AtomicBool,
    pub(super) on_closed: event_listener::Event,
    pub(super) options: PoolOptions<DB>,
}

impl<DB: Database> PoolInner<DB> {
    pub(super) fn new_arc(
        options: PoolOptions<DB>,
        connect_options: <DB::Connection as Connection>::Options,
    ) -> Arc<Self> {
        let capacity = options.max_connections as usize;

        let semaphore_capacity = if let Some(parent) = &options.parent_pool {
            assert!(options.max_connections <= parent.options().max_connections);
            assert_eq!(options.fair, parent.options().fair);
            // The child pool must steal permits from the parent
            0
        } else {
            capacity
        };

        let pool = Self {
            connect_options,
            idle_conns: ArrayQueue::new(capacity),
            semaphore: Semaphore::new(options.fair, semaphore_capacity),
            size: AtomicU32::new(0),
            num_idle: AtomicUsize::new(0),
            is_closed: AtomicBool::new(false),
            on_closed: event_listener::Event::new(),
            options,
        };

        let pool = Arc::new(pool);

        spawn_maintenance_tasks(&pool);

        pool
    }

    pub(super) fn size(&self) -> u32 {
        self.size.load(Ordering::Acquire)
    }

    pub(super) fn num_idle(&self) -> usize {
        // We don't use `self.idle_conns.len()` as it waits for the internal
        // head and tail pointers to stop changing for a moment before calculating the length,
        // which may take a long time at high levels of churn.
        //
        // By maintaining our own atomic count, we avoid that issue entirely.
        self.num_idle.load(Ordering::Acquire)
    }

    pub(super) fn is_closed(&self) -> bool {
        self.is_closed.load(Ordering::Acquire)
    }

    pub(super) fn close<'a>(self: &'a Arc<Self>) -> impl Future<Output = ()> + 'a {
        self.is_closed.store(true, Ordering::Release);
        self.on_closed.notify(usize::MAX);

        async move {
            for permits in 1..=self.options.max_connections as usize {
                // Close any currently idle connections in the pool.
                while let Some(idle) = self.idle_conns.pop() {
                    let _ = idle.live.float((*self).clone()).close().await;
                }

                if self.size() == 0 {
                    break;
                }

                // Wait for all permits to be released.
                let _permits = self.semaphore.acquire(permits).await;
            }
        }
    }

    pub(crate) fn close_event(&self) -> CloseEvent {
        CloseEvent {
            listener: (!self.is_closed()).then(|| self.on_closed.listen()),
        }
    }

    /// Attempt to pull a permit from `self.semaphore` or steal one from the parent.
    ///
    /// If we steal a permit from the parent but *don't* open a connection,
    /// it should be returned to the parent.
    async fn acquire_permit<'a>(self: &'a Arc<Self>) -> Result<SemaphoreReleaser<'a>, Error> {
        let parent = self
            .parent()
            // If we're already at the max size, we shouldn't try to steal from the parent.
            // This is just going to cause unnecessary churn in `acquire()`.
            .filter(|_| self.size() < self.options.max_connections);

        let acquire_self = self.semaphore.acquire(1).fuse();
        let mut close_event = self.close_event();

        if let Some(parent) = parent {
            let acquire_parent = parent.0.semaphore.acquire(1);
            let parent_close_event = parent.0.close_event();

            futures_util::pin_mut!(
                acquire_parent,
                acquire_self,
                close_event,
                parent_close_event
            );

            let mut poll_parent = false;

            future::poll_fn(|cx| {
                if close_event.as_mut().poll(cx).is_ready() {
                    return Poll::Ready(Err(Error::PoolClosed));
                }

                if parent_close_event.as_mut().poll(cx).is_ready() {
                    // Propagate the parent's close event to the child.
                    let _ = self.close();
                    return Poll::Ready(Err(Error::PoolClosed));
                }

                if let Poll::Ready(permit) = acquire_self.as_mut().poll(cx) {
                    return Poll::Ready(Ok(permit));
                }

                // Don't try the parent right away.
                if poll_parent {
                    acquire_parent.as_mut().poll(cx).map(Ok)
                } else {
                    poll_parent = true;
                    cx.waker().wake_by_ref();
                    Poll::Pending
                }
            })
            .await
        } else {
            close_event.do_until(acquire_self).await
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

        let permit = self.semaphore.try_acquire(1)?;

        self.pop_idle(permit).ok()
    }

    fn pop_idle<'a>(
        self: &'a Arc<Self>,
        permit: SemaphoreReleaser<'a>,
    ) -> Result<Floating<DB, Idle<DB>>, SemaphoreReleaser<'a>> {
        if let Some(idle) = self.idle_conns.pop() {
            self.num_idle.fetch_sub(1, Ordering::AcqRel);
            Ok(Floating::from_idle(idle, (*self).clone(), permit))
        } else {
            Err(permit)
        }
    }

    pub(super) fn release(&self, floating: Floating<DB, Live<DB>>) {
        // `options.after_release` is invoked by `PoolConnection::release_to_pool()`.

        let Floating { inner: idle, guard } = floating.into_idle();

        if !self.idle_conns.push(idle).is_ok() {
            panic!("BUG: connection queue overflow in release()");
        }

        // NOTE: we need to make sure we drop the permit *after* we push to the idle queue
        // don't decrease the size
        guard.release_permit();

        self.num_idle.fetch_add(1, Ordering::AcqRel);
    }

    /// Try to atomically increment the pool size for a new connection.
    pub(super) fn try_increment_size<'a>(
        self: &'a Arc<Self>,
        permit: SemaphoreReleaser<'a>,
    ) -> Result<DecrementSizeGuard<DB>, SemaphoreReleaser<'a>> {
        match self
            .size
            .fetch_update(Ordering::AcqRel, Ordering::Acquire, |size| {
                size.checked_add(1)
                    .filter(|size| size <= &self.options.max_connections)
            }) {
            // we successfully incremented the size
            Ok(_) => Ok(DecrementSizeGuard::from_permit((*self).clone(), permit)),
            // the pool is at max capacity
            Err(_) => Err(permit),
        }
    }

    pub(super) async fn acquire(self: &Arc<Self>) -> Result<Floating<DB, Live<DB>>, Error> {
        if self.is_closed() {
            return Err(Error::PoolClosed);
        }

        let deadline = Instant::now() + self.options.acquire_timeout;

        sqlx_rt::timeout(
            self.options.acquire_timeout,
            async {
                loop {
                    // Handles the close-event internally
                    let permit = self.acquire_permit().await?;


                    // First attempt to pop a connection from the idle queue.
                    let guard = match self.pop_idle(permit) {

                        // Then, check that we can use it...
                        Ok(conn) => match check_idle_conn(conn, &self.options).await {

                            // All good!
                            Ok(live) => return Ok(live),

                            // if the connection isn't usable for one reason or another,
                            // we get the `DecrementSizeGuard` back to open a new one
                            Err(guard) => guard,
                        },
                        Err(permit) => if let Ok(guard) = self.try_increment_size(permit) {
                            // we can open a new connection
                            guard
                        } else {
                            // This can happen for a child pool that's at its connection limit.
                            log::debug!("woke but was unable to acquire idle connection or open new one; retrying");
                            // If so, we're likely in the current-thread runtime if it's Tokio
                            // and so we should yield to let any spawned release_to_pool() tasks
                            // execute.
                            sqlx_rt::yield_now().await;
                            continue;
                        }
                    };

                    // Attempt to connect...
                    return self.connect(deadline, guard).await;
                }
            }
        )
            .await
            .map_err(|_| Error::PoolTimedOut)?
    }

    pub(super) async fn connect(
        self: &Arc<Self>,
        deadline: Instant,
        guard: DecrementSizeGuard<DB>,
    ) -> Result<Floating<DB, Live<DB>>, Error> {
        if self.is_closed() {
            return Err(Error::PoolClosed);
        }

        let mut backoff = Duration::from_millis(10);
        let max_backoff = deadline_as_timeout::<DB>(deadline)? / 5;

        loop {
            let timeout = deadline_as_timeout::<DB>(deadline)?;

            // result here is `Result<Result<C, Error>, TimeoutError>`
            // if this block does not return, sleep for the backoff timeout and try again
            match sqlx_rt::timeout(timeout, self.connect_options.connect()).await {
                // successfully established connection
                Ok(Ok(mut raw)) => {
                    // See comment on `PoolOptions::after_connect`
                    let meta = PoolConnectionMetadata {
                        age: Duration::ZERO,
                        idle_for: Duration::ZERO,
                    };

                    let res = if let Some(callback) = &self.options.after_connect {
                        callback(&mut raw, meta).await
                    } else {
                        Ok(())
                    };

                    match res {
                        Ok(()) => return Ok(Floating::new_live(raw, guard)),
                        Err(e) => {
                            log::error!("error returned from after_connect: {:?}", e);
                            // The connection is broken, don't try to close nicely.
                            let _ = raw.close_hard().await;

                            // Fall through to the backoff.
                        }
                    }
                }

                // an IO error while connecting is assumed to be the system starting up
                Ok(Err(Error::Io(e))) if e.kind() == std::io::ErrorKind::ConnectionRefused => (),

                // We got a transient database error, retry.
                Ok(Err(Error::Database(error))) if error.is_transient_in_connect_phase() => (),

                // Any other error while connection should immediately
                // terminate and bubble the error up
                Ok(Err(e)) => return Err(e),

                // timed out
                Err(_) => return Err(Error::PoolTimedOut),
            }

            // If the connection is refused, wait in exponentially
            // increasing steps for the server to come up,
            // capped by a factor of the remaining time until the deadline
            sqlx_rt::sleep(backoff).await;
            backoff = cmp::min(backoff * 2, max_backoff);
        }
    }

    /// Try to maintain `min_connections`, returning any errors (including `PoolTimedOut`).
    pub async fn try_min_connections(self: &Arc<Self>, deadline: Instant) -> Result<(), Error> {
        macro_rules! unwrap_or_return {
            ($expr:expr) => {
                match $expr {
                    Some(val) => val,
                    None => return Ok(()),
                }
            };
        }

        while self.size() < self.options.min_connections {
            // Don't wait for a semaphore permit.
            //
            // If no extra permits are available then we shouldn't be trying to spin up
            // connections anyway.
            let permit = unwrap_or_return!(self.semaphore.try_acquire(1));

            // We must always obey `max_connections`.
            let guard = unwrap_or_return!(self.try_increment_size(permit).ok());

            // We skip `after_release` since the connection was never provided to user code
            // besides `after_connect`, if they set it.
            self.release(self.connect(deadline, guard).await?);
        }

        Ok(())
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
                log::debug!("unable to complete `min_connections` maintenance before deadline")
            }
            Err(e) => log::debug!("error while maintaining min_connections: {:?}", e),
        }
    }
}

impl<DB: Database> Drop for PoolInner<DB> {
    fn drop(&mut self) {
        if let Some(parent) = &self.options.parent_pool {
            // Release the stolen permits.
            parent.0.semaphore.release(self.semaphore.permits());
        }
    }
}

/// Returns `true` if the connection has exceeded `options.max_lifetime` if set, `false` otherwise.
fn is_beyond_max_lifetime<DB: Database>(live: &Live<DB>, options: &PoolOptions<DB>) -> bool {
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
) -> Result<Floating<DB, Live<DB>>, DecrementSizeGuard<DB>> {
    // If the connection we pulled has expired, close the connection and
    // immediately create a new connection
    if is_beyond_max_lifetime(&conn, options) {
        return Err(conn.close().await);
    }

    if options.test_before_acquire {
        // Check that the connection is still live
        if let Err(e) = conn.ping().await {
            // an error here means the other end has hung up or we lost connectivity
            // either way we're fine to just discard the connection
            // the error itself here isn't necessarily unexpected so WARN is too strong
            log::info!("ping on idle connection returned error: {}", e);
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
                log::warn!("error from `before_acquire`: {}", error);
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
    let pool = Arc::clone(&pool);

    let period = match (pool.options.max_lifetime, pool.options.idle_timeout) {
        (Some(it), None) | (None, Some(it)) => it,

        (Some(a), Some(b)) => cmp::min(a, b),

        (None, None) => {
            if pool.options.min_connections > 0 {
                sqlx_rt::spawn(async move {
                    pool.min_connections_maintenance(None).await;
                });
            }

            return;
        }
    };

    sqlx_rt::spawn(async move {
        // Immediately cancel this task if the pool is closed.
        let _ = pool
            .close_event()
            .do_until(async {
                while !pool.is_closed() {
                    let next_run = Instant::now() + period;

                    pool.min_connections_maintenance(Some(next_run)).await;

                    if let Some(duration) = next_run.checked_duration_since(Instant::now()) {
                        // `async-std` doesn't have a `sleep_until()`
                        sqlx_rt::sleep(duration).await;
                    } else {
                        sqlx_rt::yield_now().await;
                    }

                    // Don't run the reaper right away.
                    if !pool.idle_conns.is_empty() {
                        do_reap(&pool).await;
                    }
                }
            })
            .await;
    });
}

async fn do_reap<DB: Database>(pool: &Arc<PoolInner<DB>>) {
    // reap at most the current size minus the minimum idle
    let max_reaped = pool.size().saturating_sub(pool.options.min_connections);

    // collect connections to reap
    let (reap, keep) = (0..max_reaped)
        // only connections waiting in the queue
        .filter_map(|_| pool.try_acquire())
        .partition::<Vec<_>, _>(|conn| {
            is_beyond_idle_timeout(conn, &pool.options)
                || is_beyond_max_lifetime(conn, &pool.options)
        });

    for conn in keep {
        // return valid connections to the pool first
        pool.release(conn.into_live());
    }

    for conn in reap {
        let _ = conn.close().await;
    }
}

/// RAII guard returned by `Pool::try_increment_size()` and others.
///
/// Will decrement the pool size if dropped, to avoid semantically "leaking" connections
/// (where the pool thinks it has more connections than it does).
pub(in crate::pool) struct DecrementSizeGuard<DB: Database> {
    pub(crate) pool: Arc<PoolInner<DB>>,
    cancelled: bool,
}

impl<DB: Database> DecrementSizeGuard<DB> {
    /// Create a new guard that will release a semaphore permit on-drop.
    pub fn new_permit(pool: Arc<PoolInner<DB>>) -> Self {
        Self {
            pool,
            cancelled: false,
        }
    }

    pub fn from_permit(pool: Arc<PoolInner<DB>>, mut permit: SemaphoreReleaser<'_>) -> Self {
        // here we effectively take ownership of the permit
        permit.disarm();
        Self::new_permit(pool)
    }

    /// Release the semaphore permit without decreasing the pool size.
    ///
    /// If the permit was stolen from the pool's parent, it will be returned to the child's semaphore.
    fn release_permit(self) {
        self.pool.semaphore.release(1);
        self.cancel();
    }

    pub fn cancel(mut self) {
        self.cancelled = true;
    }
}

impl<DB: Database> Drop for DecrementSizeGuard<DB> {
    fn drop(&mut self) {
        if !self.cancelled {
            self.pool.size.fetch_sub(1, Ordering::AcqRel);

            // and here we release the permit we got on construction
            self.pool.semaphore.release(1);
        }
    }
}
