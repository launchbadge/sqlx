use super::connection::{Floating, Idle, Live};
use crate::connection::ConnectOptions;
use crate::connection::Connection;
use crate::database::Database;
use crate::error::Error;
use crate::pool::{deadline_as_timeout, PoolOptions};
use crossbeam_queue::ArrayQueue;

use futures_intrusive::sync::{Semaphore, SemaphoreReleaser};

use std::cmp;
use std::mem;
use std::ptr;
use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};
use std::sync::Arc;

use std::time::{Duration, Instant};

/// Ihe number of permits to release to wake all waiters, such as on `SharedPool::close()`.
///
/// This should be large enough to realistically wake all tasks waiting on the pool without
/// potentially overflowing the permits count in the semaphore itself.
const WAKE_ALL_PERMITS: usize = usize::MAX / 2;

pub(crate) struct SharedPool<DB: Database> {
    pub(super) connect_options: <DB::Connection as Connection>::Options,
    pub(super) idle_conns: ArrayQueue<Idle<DB>>,
    pub(super) semaphore: Semaphore,
    pub(super) size: AtomicU32,
    is_closed: AtomicBool,
    pub(super) options: PoolOptions<DB>,
}

impl<DB: Database> SharedPool<DB> {
    pub(super) fn new_arc(
        options: PoolOptions<DB>,
        connect_options: <DB::Connection as Connection>::Options,
    ) -> Arc<Self> {
        let capacity = options.max_connections as usize;

        // ensure the permit count won't overflow if we release `WAKE_ALL_PERMITS`
        // this assert should never fire on 64-bit targets as `max_connections` is a u32
        let _ = capacity
            .checked_add(WAKE_ALL_PERMITS)
            .expect("max_connections exceeds max capacity of the pool");

        let pool = Self {
            connect_options,
            idle_conns: ArrayQueue::new(capacity),
            semaphore: Semaphore::new(options.fair, capacity),
            size: AtomicU32::new(0),
            is_closed: AtomicBool::new(false),
            options,
        };

        let pool = Arc::new(pool);

        spawn_reaper(&pool);

        pool
    }

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
        let already_closed = self.is_closed.swap(true, Ordering::AcqRel);

        if !already_closed {
            // if we were the one to mark this closed, release enough permits to wake all waiters
            // we can't just do `usize::MAX` because that would overflow
            // and we can't do this more than once cause that would _also_ overflow
            self.semaphore.release(WAKE_ALL_PERMITS);
        }

        // wait for all permits to be released
        let _permits = self
            .semaphore
            .acquire(WAKE_ALL_PERMITS + (self.options.max_connections as usize))
            .await;

        while let Some(idle) = self.idle_conns.pop() {
            let _ = idle.live.float(self).close().await;
        }
    }

    #[inline]
    pub(super) fn try_acquire(&self) -> Option<Floating<'_, Idle<DB>>> {
        if self.is_closed() {
            return None;
        }

        let permit = self.semaphore.try_acquire(1)?;
        self.pop_idle(permit).ok()
    }

    fn pop_idle<'a>(
        &'a self,
        permit: SemaphoreReleaser<'a>,
    ) -> Result<Floating<'a, Idle<DB>>, SemaphoreReleaser<'a>> {
        if let Some(idle) = self.idle_conns.pop() {
            Ok(Floating::from_idle(idle, self, permit))
        } else {
            Err(permit)
        }
    }

    pub(super) fn release(&self, mut floating: Floating<'_, Live<DB>>) {
        if let Some(test) = &self.options.after_release {
            if !test(&mut floating.raw) {
                // drop the connection and do not return it to the pool
                return;
            }
        }

        let Floating { inner: idle, guard } = floating.into_idle();

        if !self.idle_conns.push(idle).is_ok() {
            panic!("BUG: connection queue overflow in release()");
        }

        // NOTE: we need to make sure we drop the permit *after* we push to the idle queue
        // don't decrease the size
        guard.release_permit();
    }

    /// Try to atomically increment the pool size for a new connection.
    ///
    /// Returns `None` if we are at max_connections or if the pool is closed.
    pub(super) fn try_increment_size<'a>(
        &'a self,
        permit: SemaphoreReleaser<'a>,
    ) -> Result<DecrementSizeGuard<'a>, SemaphoreReleaser<'a>> {
        match self
            .size
            .fetch_update(Ordering::AcqRel, Ordering::Acquire, |size| {
                size.checked_add(1)
                    .filter(|size| size <= &self.options.max_connections)
            }) {
            // we successfully incremented the size
            Ok(_) => Ok(DecrementSizeGuard::from_permit(self, permit)),
            // the pool is at max capacity
            Err(_) => Err(permit),
        }
    }

    #[allow(clippy::needless_lifetimes)]
    pub(super) async fn acquire<'s>(&'s self) -> Result<Floating<'s, Live<DB>>, Error> {
        if self.is_closed() {
            return Err(Error::PoolClosed);
        }

        let deadline = Instant::now() + self.options.connect_timeout;

        sqlx_rt::timeout(
            self.options.connect_timeout,
            async {
                loop {
                    let permit = self.semaphore.acquire(1).await;

                    if self.is_closed() {
                        return Err(Error::PoolClosed);
                    }

                    // First attempt to pop a connection from the idle queue.
                    let guard = match self.pop_idle(permit) {

                        // Then, check that we can use it...
                        Ok(conn) => match check_conn(conn, &self.options).await {

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
                            log::debug!("woke but was unable to acquire idle connection or open new one; retrying");
                            continue;
                        }
                    };

                    // Attempt to connect...
                    return self.connection(deadline, guard).await;
                }
            }
        )
            .await
            .map_err(|_| Error::PoolTimedOut)?
    }

    pub(super) async fn connection<'s>(
        &'s self,
        deadline: Instant,
        guard: DecrementSizeGuard<'s>,
    ) -> Result<Floating<'s, Live<DB>>, Error> {
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
                    if let Some(callback) = &self.options.after_connect {
                        callback(&mut raw).await?;
                    }

                    return Ok(Floating::new_live(raw, guard));
                }

                // an IO error while connecting is assumed to be the system starting up
                Ok(Err(Error::Io(e))) if e.kind() == std::io::ErrorKind::ConnectionRefused => (),

                // TODO: Handle other database "boot period"s

                // [postgres] the database system is starting up
                // TODO: Make this check actually check if this is postgres
                Ok(Err(Error::Database(error))) if error.code().as_deref() == Some("57P03") => (),

                // Any other error while connection should immediately
                // terminate and bubble the error up
                Ok(Err(e)) => return Err(e),

                // timed out
                Err(_) => return Err(Error::PoolTimedOut),
            }

            // If the connection is refused wait in exponentially
            // increasing steps for the server to come up,
            // capped by a factor of the remaining time until the deadline
            sqlx_rt::sleep(backoff).await;
            backoff = cmp::min(backoff * 2, max_backoff);
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
) -> Result<Floating<'s, Live<DB>>, DecrementSizeGuard<'s>> {
    // If the connection we pulled has expired, close the connection and
    // immediately create a new connection
    if is_beyond_lifetime(&conn, options) {
        // we're closing the connection either way
        // close the connection but don't really care about the result
        return Err(conn.close().await);
    } else if options.test_before_acquire {
        // Check that the connection is still live
        if let Err(e) = conn.ping().await {
            // an error here means the other end has hung up or we lost connectivity
            // either way we're fine to just discard the connection
            // the error itself here isn't necessarily unexpected so WARN is too strong
            log::info!("ping on idle connection returned error: {}", e);
            // connection is broken so don't try to close nicely
            return Err(conn.close().await);
        }
    } else if let Some(test) = &options.before_acquire {
        match test(&mut conn.live.raw).await {
            Ok(false) => {
                // connection was rejected by user-defined hook
                return Err(conn.close().await);
            }

            Err(error) => {
                log::info!("in `before_acquire`: {}", error);
                return Err(conn.close().await);
            }

            Ok(true) => {}
        }
    }

    // No need to re-connect; connection is alive or we don't care
    Ok(conn.into_live())
}

/// if `max_lifetime` or `idle_timeout` is set, spawn a task that reaps senescent connections
fn spawn_reaper<DB: Database>(pool: &Arc<SharedPool<DB>>) {
    let period = match (pool.options.max_lifetime, pool.options.idle_timeout) {
        (Some(it), None) | (None, Some(it)) => it,

        (Some(a), Some(b)) => cmp::min(a, b),

        (None, None) => return,
    };

    let pool = Arc::clone(&pool);

    sqlx_rt::spawn(async move {
        while !pool.is_closed() {
            if !pool.idle_conns.is_empty() {
                do_reap(&pool).await;
            }
            sqlx_rt::sleep(period).await;
        }
    });
}

async fn do_reap<DB: Database>(pool: &SharedPool<DB>) {
    // reap at most the current size minus the minimum idle
    let max_reaped = pool.size().saturating_sub(pool.options.min_connections);

    // collect connections to reap
    let (reap, keep) = (0..max_reaped)
        // only connections waiting in the queue
        .filter_map(|_| pool.try_acquire())
        .partition::<Vec<_>, _>(|conn| {
            is_beyond_idle(conn, &pool.options) || is_beyond_lifetime(conn, &pool.options)
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
pub(in crate::pool) struct DecrementSizeGuard<'a> {
    size: &'a AtomicU32,
    semaphore: &'a Semaphore,
    dropped: bool,
}

impl<'a> DecrementSizeGuard<'a> {
    /// Create a new guard that will release a semaphore permit on-drop.
    pub fn new_permit<DB: Database>(pool: &'a SharedPool<DB>) -> Self {
        Self {
            size: &pool.size,
            semaphore: &pool.semaphore,
            dropped: false,
        }
    }

    pub fn from_permit<DB: Database>(
        pool: &'a SharedPool<DB>,
        mut permit: SemaphoreReleaser<'a>,
    ) -> Self {
        // here we effectively take ownership of the permit
        permit.disarm();
        Self::new_permit(pool)
    }

    /// Return `true` if the internal references point to the same fields in `SharedPool`.
    pub fn same_pool<DB: Database>(&self, pool: &'a SharedPool<DB>) -> bool {
        ptr::eq(self.size, &pool.size)
    }

    /// Release the semaphore permit without decreasing the pool size.
    fn release_permit(self) {
        self.semaphore.release(1);
        self.cancel();
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

        // and here we release the permit we got on construction
        self.semaphore.release(1);
    }
}
