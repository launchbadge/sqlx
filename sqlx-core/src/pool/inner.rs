use std::cmp;
use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};
use std::sync::Arc;
use std::time::Instant;

use crossbeam_queue::{ArrayQueue, SegQueue};
use futures_channel::oneshot::{channel, Sender};

use crate::runtime::{sleep, spawn, timeout, yield_now};
use super::{Idle, Live, Options};
use crate::{
    connection::{Connect, Connection},
    error::Error,
};

pub(super) struct SharedPool<C> {
    url: String,
    idle: ArrayQueue<Idle<C>>,
    waiters: SegQueue<Sender<Live<C>>>,
    size: AtomicU32,
    is_closed: AtomicBool,
    options: Options,
}

impl<C> SharedPool<C>
where
    C: Connection + Connect<Connection = C>,
{
    pub(super) async fn new_arc(url: &str, options: Options) -> crate::Result<Arc<Self>> {
        let pool = Arc::new(Self {
            url: url.to_owned(),
            idle: ArrayQueue::new(options.max_size as usize),
            waiters: SegQueue::new(),
            size: AtomicU32::new(0),
            is_closed: AtomicBool::new(false),
            options,
        });

        // If a minimum size was configured for the pool,
        // establish N connections
        // TODO: Should we do this in the background?
        for _ in 0..pool.options.min_size {
            let live = pool
                .eventually_connect(Instant::now() + pool.options.connect_timeout)
                .await?;

            // Ignore error here, we are capping this loop by min_size which we
            // already should make sure is less than max_size
            let _ = pool.idle.push(Idle {
                live,
                since: Instant::now(),
            });
        }

        spawn_reaper(&pool);

        Ok(pool)
    }

    pub fn options(&self) -> &Options {
        &self.options
    }

    pub(super) fn url(&self) -> &str {
        &self.url
    }

    pub(super) fn size(&self) -> u32 {
        self.size.load(Ordering::Acquire)
    }

    pub(super) fn num_idle(&self) -> usize {
        // NOTE: This is very expensive
        self.waiters.len()
    }

    pub(super) fn is_closed(&self) -> bool {
        self.is_closed.load(Ordering::Acquire)
    }

    pub(super) async fn close(&self) {
        self.is_closed.store(true, Ordering::Release);

        while self.size.load(Ordering::Acquire) > 0 {
            // don't block on the receiver because we own one Sender so it should never return
            // `None`; a `select!()` would also work but that produces more complicated code
            // and a timeout isn't necessarily appropriate
            while let Ok(idle) = self.idle.pop() {
                idle.close().await;
                self.size.fetch_sub(1, Ordering::AcqRel);
            }

            yield_now().await
        }
    }

    #[inline]
    pub(super) fn try_acquire(&self) -> Option<Live<C>> {
        if self.is_closed.load(Ordering::Acquire) {
            return None;
        }

        Some(self.idle.pop().ok()?.live)
    }

    pub(super) fn release(&self, mut live: Live<C>) {
        // Try waiters in (FIFO) order until one is still waiting ..
        while let Ok(waiter) = self.waiters.pop() {
            live = match waiter.send(live) {
                // successfully released
                Ok(()) => return,

                Err(live) => live,
            };
        }

        // .. if there were no waiters still waiting, just push the connection
        // back to the idle queue
        let _ = self.idle.push(Idle {
            live,
            since: Instant::now(),
        });
    }

    pub(super) async fn acquire(&self) -> crate::Result<Live<C>> {
        let start = Instant::now();
        let deadline = start + self.options.connect_timeout;

        // Unless the pool has been closed ...
        while !self.is_closed.load(Ordering::Acquire) {
            // Attempt to immediately acquire a connection. This will return Some
            // if there is an idle connection in our channel.
            if let Some(idle) = self.idle.pop().ok() {
                if let Some(live) = check_live(idle.live, &self.options).await {
                    return Ok(live);
                }
            }

            let size = self.size.load(Ordering::Acquire);

            if size >= self.options.max_size {
                // Too many open connections
                // Wait until one is available
                let (tx, rx) = channel();

                self.waiters.push(tx);

                // get the time between the deadline and now and use that as our timeout
                let until = deadline
                    .checked_duration_since(Instant::now())
                    .ok_or(Error::PoolTimedOut(None))?;

                // don't sleep forever
                let live = match timeout(until, rx).await {
                    // A connection was returned to the pool
                    Ok(Ok(live)) => live,

                    // Pool dropped without dropping waiter
                    Ok(Err(_)) => unreachable!(),

                    // Timed out waiting for a connection
                    // Error is not forwarded as its useless context
                    Err(_) => {
                        return Err(Error::PoolTimedOut(None));
                    }
                };

                // If pool was closed while waiting for a connection,
                // release the connection
                if self.is_closed.load(Ordering::Acquire) {
                    live.close().await;
                    self.size.fetch_sub(1, Ordering::AcqRel);

                    return Err(Error::PoolClosed);
                }

                match check_live(live, &self.options).await {
                    Some(live) => return Ok(live),

                    // Need to re-connect
                    None => {}
                }
            } else if self.size.compare_and_swap(size, size + 1, Ordering::AcqRel) != size {
                // size was incremented while we compared it just above
                continue;
            }

            // pool has slots available; open a new connection
            match self.connect(deadline).await {
                Ok(Some(conn)) => return Ok(conn),
                // [size] is internally decremented on _retry_ and _error_
                Ok(None) => continue,
                Err(e) => return Err(e),
            }
        }

        Err(Error::PoolClosed)
    }

    async fn eventually_connect(&self, deadline: Instant) -> crate::Result<Live<C>> {
        loop {
            // [connect] will raise an error when past deadline
            // [connect] returns None if its okay to retry
            if let Some(conn) = self.connect(deadline).await? {
                return Ok(conn);
            }
        }
    }

    async fn connect(&self, deadline: Instant) -> crate::Result<Option<Live<C>>> {
        // FIXME: Code between `-` is duplicate with [acquire]
        // ---------------------------------

        // get the time between the deadline and now and use that as our timeout
        let until = deadline
            .checked_duration_since(Instant::now())
            .ok_or(Error::PoolTimedOut(None))?;

        // If pool was closed while waiting for a connection,
        // release the connection
        if self.is_closed.load(Ordering::Acquire) {
            self.size.fetch_sub(1, Ordering::AcqRel); // ?

            return Err(Error::PoolClosed);
        }

        // ---------------------------------

        // result here is `Result<Result<C, Error>, TimeoutError>`
        match timeout(until, C::connect(&self.url)).await {
            // successfully established connection
            Ok(Ok(raw)) => {
                Ok(Some(Live {
                    raw,
                    // remember when it was created so we can expire it
                    // if there is a [max_lifetime] set
                    created: Instant::now(),
                }))
            }

            // IO error while connecting, this should definitely be logged
            // and we should attempt to retry
            Ok(Err(crate::Error::Io(e))) => {
                log::warn!("error establishing a connection: {}", e);

                Ok(None)
            }

            // Any other error while connection should immediately
            // terminate and bubble the error up
            Ok(Err(e)) => Err(e),

            // timed out
            Err(e) => {
                self.size.fetch_sub(1, Ordering::AcqRel); // ?
                Err(Error::PoolTimedOut(Some(Box::new(e))))
            }
        }
    }
}

impl<C> Idle<C>
where
    C: Connection,
{
    async fn close(self) {
        self.live.close().await;
    }
}

impl<C> Live<C>
where
    C: Connection,
{
    async fn close(self) {
        let _ = self.raw.close().await;
    }
}

// NOTE: Function names here are bizzare. Helpful help would be appreciated.

fn is_beyond_lifetime<C>(live: &Live<C>, options: &Options) -> bool {
    // check if connection was within max lifetime (or not set)
    options
        .max_lifetime
        .map_or(false, |max| live.created.elapsed() > max)
}

fn is_beyond_idle<C>(idle: &Idle<C>, options: &Options) -> bool {
    // if connection wasn't idle too long (or not set)
    options
        .idle_timeout
        .map_or(false, |timeout| idle.since.elapsed() > timeout)
}

async fn check_live<C>(mut live: Live<C>, options: &Options) -> Option<Live<C>>
where
    C: Connection,
{
    // If the connection we pulled has expired, close the connection and
    // immediately create a new connection
    if is_beyond_lifetime(&live, options) {
        // close the connection but don't really care about the result
        let _ = live.close().await;
    } else if options.test_on_acquire {
        // TODO: Check on acquire should be a configuration setting
        // Check that the connection is still live
        match live.raw.ping().await {
            // Connection still seems to respond
            Ok(_) => return Some(live),

            // an error here means the other end has hung up or we lost connectivity
            // either way we're fine to just discard the connection
            // the error itself here isn't necessarily unexpected so WARN is too strong
            Err(e) => log::info!("ping on idle connection returned error: {}", e),
        }

        // make sure the idle connection is gone explicitly before we open one
        // this will close the resources for the stream on our side
        drop(live);
    } else {
        // No need to re-connect
        return Some(live);
    }

    None
}

/// if `max_lifetime` or `idle_timeout` is set, spawn a task that reaps senescent connections
fn spawn_reaper<C>(pool: &Arc<SharedPool<C>>)
where
    C: Connection,
{
    let period = match (pool.options.max_lifetime, pool.options.idle_timeout) {
        (Some(it), None) | (None, Some(it)) => it,

        (Some(a), Some(b)) => cmp::min(a, b),

        (None, None) => return,
    };

    let pool = Arc::clone(&pool);

    spawn(async move {
        while !pool.is_closed.load(Ordering::Acquire) {
            // reap at most the current size minus the minimum idle
            let max_reaped = pool
                .size
                .load(Ordering::Acquire)
                .saturating_sub(pool.options.min_size);

            // collect connections to reap
            let (reap, keep) = (0..max_reaped)
                // only connections waiting in the queue
                .filter_map(|_| pool.idle.pop().ok())
                .partition::<Vec<_>, _>(|conn| {
                    is_beyond_idle(conn, &pool.options)
                        || is_beyond_lifetime(&conn.live, &pool.options)
                });

            for conn in keep {
                // return these connections to the pool first
                pool.idle.push(conn).expect("unreachable: pool overflowed");
            }

            for conn in reap {
                conn.close().await;
                pool.size.fetch_sub(1, Ordering::AcqRel);
            }

            sleep(period).await;
        }
    });
}
