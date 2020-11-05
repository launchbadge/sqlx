use super::connection::{Floating, Idle, Live};
use crate::connection::ConnectOptions;
use crate::connection::Connection;
use crate::database::Database;
use crate::error::Error;
use crate::pool::{deadline_as_timeout, PoolOptions};
use crossbeam_queue::{ArrayQueue, SegQueue};
use futures_core::task::{Poll, Waker};
use futures_util::future;
use sqlx_rt::{sleep, spawn, timeout};
use std::cmp;
use std::mem;
use std::ptr;
use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};
use std::sync::Arc;
use std::task::Context;
use std::time::Instant;

pub(crate) struct SharedPool<DB: Database> {
    pub(super) connect_options: <DB::Connection as Connection>::Options,
    pub(super) idle_conns: ArrayQueue<Idle<DB>>,
    waiters: SegQueue<Arc<Waiter>>,
    pub(super) size: AtomicU32,
    is_closed: AtomicBool,
    pub(super) options: PoolOptions<DB>,
}

impl<DB: Database> SharedPool<DB> {
    pub(super) fn size(&self) -> u32 {
        self.size.load(Ordering::Acquire)
    }

    pub(super) fn num_idle(&self) -> usize {
        // NOTE: This is very expensive
        self.idle_conns.len()
    }

    pub(super) fn is_closed(&self) -> bool {
        self.is_closed.load(Ordering::Acquire)
    }

    pub(super) async fn close(&self) {
        self.is_closed.store(true, Ordering::Release);
        while let Ok(waker) = self.waiters.pop() {
            waker.wake();
        }

        // ensure we wait until the pool is actually closed
        while self.size() > 0 {
            let _ = self
                .idle_conns
                .pop()
                .map(|idle| Floating::from_idle(idle, self));

            // yield to avoid starving the executor
            sqlx_rt::yield_now().await;
        }
    }

    #[inline]
    pub(super) fn try_acquire(&self) -> Option<Floating<'_, Live<DB>>> {
        // don't cut in line
        if self.options.fair && !self.waiters.is_empty() {
            return None;
        }
        Some(self.pop_idle()?.into_live())
    }

    fn pop_idle(&self) -> Option<Floating<'_, Idle<DB>>> {
        if self.is_closed.load(Ordering::Acquire) {
            return None;
        }

        Some(Floating::from_idle(self.idle_conns.pop().ok()?, self))
    }

    pub(super) fn release(&self, mut floating: Floating<'_, Live<DB>>) {
        if let Some(test) = &self.options.after_release {
            if !test(&mut floating.raw) {
                // drop the connection and do not return to the pool
                return;
            }
        }

        self.idle_conns
            .push(floating.into_idle().into_leakable())
            .expect("BUG: connection queue overflow in release()");

        if let Ok(waker) = self.waiters.pop() {
            waker.wake();
        }
    }

    /// Try to atomically increment the pool size for a new connection.
    ///
    /// Returns `None` if we are at max_connections or if the pool is closed.
    pub(super) fn try_increment_size(&self) -> Option<DecrementSizeGuard<'_>> {
        if self.is_closed() {
            return None;
        }

        let mut size = self.size();

        while size < self.options.max_connections {
            let new_size = self.size.compare_and_swap(size, size + 1, Ordering::AcqRel);

            if new_size == size {
                return Some(DecrementSizeGuard::new(self));
            }

            size = new_size;
        }

        None
    }

    /// Wait for a connection, if either `size` drops below `max_connections` so we can
    /// open a new connection, or if an idle connection is returned to the pool.
    ///
    /// Returns an error if `deadline` elapses before we are woken.
    async fn wait_for_conn(&self, deadline: Instant) -> Result<(), Error> {
        if self.is_closed() {
            return Err(Error::PoolClosed);
        }

        let mut waiter = None;

        timeout(
            deadline_as_timeout::<DB>(deadline)?,
            // `poll_fn` gets us easy access to a `Waker` that we can push to our queue
            future::poll_fn(|cx| -> Poll<()> {
                let waiter = waiter.get_or_insert_with(|| {
                    let waiter = Waiter::new(cx);
                    self.waiters.push(waiter.clone());
                    waiter
                });

                if waiter.is_woken() {
                    Poll::Ready(())
                } else {
                    Poll::Pending
                }
            }),
        )
        .await
        .map_err(|_| Error::PoolTimedOut)
    }

    pub(super) fn new_arc(
        options: PoolOptions<DB>,
        connect_options: <DB::Connection as Connection>::Options,
    ) -> Arc<Self> {
        let pool = Self {
            connect_options,
            idle_conns: ArrayQueue::new(options.max_connections as usize),
            waiters: SegQueue::new(),
            size: AtomicU32::new(0),
            is_closed: AtomicBool::new(false),
            options,
        };

        let pool = Arc::new(pool);

        spawn_reaper(&pool);

        pool
    }

    #[allow(clippy::needless_lifetimes)]
    pub(super) async fn acquire<'s>(&'s self) -> Result<Floating<'s, Live<DB>>, Error> {
        let start = Instant::now();
        let deadline = start + self.options.connect_timeout;
        let mut waited = !self.options.fair;

        // Unless the pool has been closed ...
        while !self.is_closed() {
            // Don't cut in line
            if waited || self.waiters.is_empty() {
                // Attempt to immediately acquire a connection. This will return Some
                // if there is an idle connection in our channel.
                if let Some(conn) = self.pop_idle() {
                    if let Some(live) = check_conn(conn, &self.options).await {
                        return Ok(live);
                    }
                }
            }

            if let Some(guard) = self.try_increment_size() {
                // pool has slots available; open a new connection
                match self.connection(deadline, guard).await {
                    Ok(Some(conn)) => return Ok(conn),
                    // [size] is internally decremented on _retry_ and _error_
                    Ok(None) => continue,
                    Err(e) => return Err(e),
                }
            }

            // Wait for a connection to become available (or we are allowed to open a new one)
            // Returns an error if `deadline` passes
            self.wait_for_conn(deadline).await?;

            waited = true;
        }

        Err(Error::PoolClosed)
    }

    pub(super) async fn connection<'s>(
        &'s self,
        deadline: Instant,
        guard: DecrementSizeGuard<'s>,
    ) -> Result<Option<Floating<'s, Live<DB>>>, Error> {
        if self.is_closed() {
            return Err(Error::PoolClosed);
        }

        let timeout = super::deadline_as_timeout::<DB>(deadline)?;

        // result here is `Result<Result<C, Error>, TimeoutError>`
        match sqlx_rt::timeout(timeout, self.connect_options.connect()).await {
            // successfully established connection
            Ok(Ok(mut raw)) => {
                if let Some(callback) = &self.options.after_connect {
                    callback(&mut raw).await?;
                }

                Ok(Some(Floating::new_live(raw, guard)))
            }

            // an IO error while connecting is assumed to be the system starting up
            Ok(Err(Error::Io(_))) => Ok(None),

            // TODO: Handle other database "boot period"s

            // [postgres] the database system is starting up
            // TODO: Make this check actually check if this is postgres
            Ok(Err(Error::Database(error))) if error.code().as_deref() == Some("57P03") => Ok(None),

            // Any other error while connection should immediately
            // terminate and bubble the error up
            Ok(Err(e)) => Err(e),

            // timed out
            Err(_) => Err(Error::PoolTimedOut),
        }
    }
}

// NOTE: Function names here are bizzare. Helpful help would be appreciated.

fn is_beyond_lifetime<DB: Database>(live: &Live<DB>, options: &PoolOptions<DB>) -> bool {
    // check if connection was within max lifetime (or not set)
    options
        .max_lifetime
        .map_or(false, |max| live.created.elapsed() > max)
}

fn is_beyond_idle<DB: Database>(idle: &Idle<DB>, options: &PoolOptions<DB>) -> bool {
    // if connection wasn't idle too long (or not set)
    options
        .idle_timeout
        .map_or(false, |timeout| idle.since.elapsed() > timeout)
}

async fn check_conn<'s: 'p, 'p, DB: Database>(
    mut conn: Floating<'s, Idle<DB>>,
    options: &'p PoolOptions<DB>,
) -> Option<Floating<'s, Live<DB>>> {
    // If the connection we pulled has expired, close the connection and
    // immediately create a new connection
    if is_beyond_lifetime(&conn, options) {
        // we're closing the connection either way
        // close the connection but don't really care about the result
        let _ = conn.close().await;
        return None;
    } else if options.test_before_acquire {
        // Check that the connection is still live
        if let Err(e) = conn.ping().await {
            // an error here means the other end has hung up or we lost connectivity
            // either way we're fine to just discard the connection
            // the error itself here isn't necessarily unexpected so WARN is too strong
            log::info!("ping on idle connection returned error: {}", e);
            // connection is broken so don't try to close nicely
            return None;
        }
    } else if let Some(test) = &options.before_acquire {
        match test(&mut conn.live.raw).await {
            Ok(false) => {
                // connection was rejected by user-defined hook
                return None;
            }

            Err(error) => {
                log::info!("in `before_acquire`: {}", error);
                return None;
            }

            Ok(true) => {}
        }
    }

    // No need to re-connect; connection is alive or we don't care
    Some(conn.into_live())
}

/// if `max_lifetime` or `idle_timeout` is set, spawn a task that reaps senescent connections
fn spawn_reaper<DB: Database>(pool: &Arc<SharedPool<DB>>) {
    let period = match (pool.options.max_lifetime, pool.options.idle_timeout) {
        (Some(it), None) | (None, Some(it)) => it,

        (Some(a), Some(b)) => cmp::min(a, b),

        (None, None) => return,
    };

    let pool = Arc::clone(&pool);

    spawn(async move {
        while !pool.is_closed.load(Ordering::Acquire) {
            // reap at most the current size minus the minimum idle
            let max_reaped = pool.size().saturating_sub(pool.options.min_connections);

            // collect connections to reap
            let (reap, keep) = (0..max_reaped)
                // only connections waiting in the queue
                .filter_map(|_| pool.pop_idle())
                .partition::<Vec<_>, _>(|conn| {
                    is_beyond_idle(conn, &pool.options) || is_beyond_lifetime(conn, &pool.options)
                });

            for conn in keep {
                // return these connections to the pool first
                pool.idle_conns
                    .push(conn.into_leakable())
                    .expect("BUG: connection queue overflow in spawn_reaper");
            }

            for conn in reap {
                let _ = conn.close().await;
            }

            sleep(period).await;
        }
    });
}

/// RAII guard returned by `Pool::try_increment_size()` and others.
///
/// Will decrement the pool size if dropped, to avoid semantically "leaking" connections
/// (where the pool thinks it has more connections than it does).
pub(in crate::pool) struct DecrementSizeGuard<'a> {
    size: &'a AtomicU32,
    waiters: &'a SegQueue<Arc<Waiter>>,
    dropped: bool,
}

impl<'a> DecrementSizeGuard<'a> {
    pub fn new<DB: Database>(pool: &'a SharedPool<DB>) -> Self {
        Self {
            size: &pool.size,
            waiters: &pool.waiters,
            dropped: false,
        }
    }

    /// Return `true` if the internal references point to the same fields in `SharedPool`.
    pub fn same_pool<DB: Database>(&self, pool: &'a SharedPool<DB>) -> bool {
        ptr::eq(self.size, &pool.size) && ptr::eq(self.waiters, &pool.waiters)
    }

    pub fn cancel(self) {
        mem::forget(self);
    }
}

impl Drop for DecrementSizeGuard<'_> {
    fn drop(&mut self) {
        assert!(!self.dropped, "double-dropped!");
        self.dropped = true;
        self.size.fetch_sub(1, Ordering::SeqCst);
        if let Ok(waker) = self.waiters.pop() {
            waker.wake();
        }
    }
}

struct Waiter {
    woken: AtomicBool,
    waker: Waker,
}

impl Waiter {
    fn new(cx: &mut Context<'_>) -> Arc<Self> {
        Arc::new(Self {
            woken: AtomicBool::new(false),
            waker: cx.waker().clone(),
        })
    }

    fn wake(&self) {
        self.woken.store(true, Ordering::Release);
        self.waker.wake_by_ref();
    }

    fn is_woken(&self) -> bool {
        self.woken.load(Ordering::Acquire)
    }
}
