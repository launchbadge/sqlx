use std::cmp;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Instant;

use crate::runtime::{sleep, spawn};
use crate::{
    connection::{Connect, Connection},
    error::Error,
};

use super::conn::{Floating, Idle, Live};
use super::queue::ConnectionQueue;
use super::size::{IncreaseGuard, PoolSize};
use super::Options;

pub(super) struct SharedPool<C> {
    url: String,
    queue: ConnectionQueue<C>,
    pub(super) size: PoolSize,
    is_closed: AtomicBool,
    options: Options,
}

impl<C> SharedPool<C>
where
    C: Connection + Connect<Connection = C>,
{
    pub(super) async fn new_arc(url: &str, options: Options) -> crate::Result<Arc<Self>> {
        let mut pool = Self {
            url: url.to_owned(),
            queue: ConnectionQueue::new(options.max_size),
            size: PoolSize::new(options.max_size),
            is_closed: AtomicBool::new(false),
            options,
        };

        pool.init_min_connections().await?;

        let pool = Arc::new(pool);

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
        self.size.current()
    }

    pub(super) fn num_idle(&self) -> usize {
        // NOTE: This is very expensive
        self.queue.num_idle()
    }

    pub(super) fn is_closed(&self) -> bool {
        self.is_closed.load(Ordering::Acquire)
    }

    pub(super) async fn close(&self) {
        self.is_closed.store(true, Ordering::Release);
        self.queue.dump(&self.size).await;
    }

    #[inline]
    pub(super) fn try_acquire(&self) -> Option<Floating<Live<C>>> {
        if self.is_closed.load(Ordering::Acquire) {
            return None;
        }

        Some(self.queue.try_pop(&self.size)?.into_live())
    }

    pub(super) fn release(&self, floating: Floating<Live<C>>) {
        self.queue.push(floating.into_idle());
    }

    pub(super) async fn acquire<'s>(&'s self) -> crate::Result<Floating<'s, Live<C>>> {
        let start = Instant::now();
        let deadline = start + self.options.connect_timeout;

        // Unless the pool has been closed ...
        while !self.is_closed() {
            // Attempt to immediately acquire a connection. This will return Some
            // if there is an idle connection in our channel.
            if let Some(conn) = self.queue.try_pop(&self.size) {
                if let Some(live) = check_conn(conn, &self.options).await {
                    return Ok(live);
                }
            }

            if let Some(guard) = self.size.try_increase() {
                // pool has slots available; open a new connection
                match self.connect(deadline, guard).await {
                    Ok(Some(conn)) => return Ok(conn),
                    // [size] is internally decremented on _retry_ and _error_
                    Ok(None) => continue,
                    Err(e) => return Err(e),
                }
            }

            let idle = self.queue.pop(&self.size, deadline).await?;

            // If pool was closed while waiting for a connection,
            // release the connection
            if self.is_closed.load(Ordering::Acquire) {
                // closing is a courtesy, we don't care if it succeeded
                let _ = idle.close().await;
                return Err(Error::PoolClosed);
            }

            match check_conn(idle, &self.options).await {
                Some(live) => return Ok(live),

                // Need to re-connect
                None => {}
            }
        }

        Err(Error::PoolClosed)
    }

    // takes `&mut self` so this can only be called during init
    async fn init_min_connections(&mut self) -> crate::Result<()> {
        for _ in 0..self.options.min_size {
            let deadline = Instant::now() + self.options.connect_timeout;

            // this guard will prevent us from exceeding `max_size`
            while let Some(guard) = self.size.try_increase() {
                // [connect] will raise an error when past deadline
                // [connect] returns None if its okay to retry
                if let Some(conn) = self.connect(deadline, guard).await? {
                    self.queue.push(conn.into_idle());
                }
            }
        }

        Ok(())
    }

    async fn connect<'s>(
        &'s self,
        deadline: Instant,
        guard: IncreaseGuard<'s>,
    ) -> crate::Result<Option<Floating<'s, Live<C>>>> {
        if self.is_closed() {
            return Err(Error::PoolClosed);
        }

        let timeout = super::deadline_as_timeout(deadline)?;

        // result here is `Result<Result<C, Error>, TimeoutError>`
        match crate::runtime::timeout(timeout, C::connect(&self.url)).await {
            // successfully established connection
            Ok(Ok(raw)) => {
                guard.commit();
                Ok(Some(Floating::new_live(raw, &self.size)))
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
            Err(e) => Err(Error::PoolTimedOut(Some(Box::new(e)))),
        }
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

async fn check_conn<'s, C>(
    mut conn: Floating<'s, Idle<C>>,
    options: &Options,
) -> Option<Floating<'s, Live<C>>>
where
    C: Connection,
{
    // If the connection we pulled has expired, close the connection and
    // immediately create a new connection
    if is_beyond_lifetime(&conn, options) {
        // we're closing the connection either way
        // close the connection but don't really care about the result
        let _ = conn.close().await;
        return None;
    } else if options.test_on_acquire {
        // TODO: Check on acquire should be a configuration setting
        // Check that the connection is still live
        if let Err(e) = conn.ping().await {
            // an error here means the other end has hung up or we lost connectivity
            // either way we're fine to just discard the connection
            // the error itself here isn't necessarily unexpected so WARN is too strong
            log::info!("ping on idle connection returned error: {}", e);
            // connection is broken so don't try to close nicely
            return None;
        }
    }

    // No need to re-connect; connection is alive or we don't care
    Some(conn.into_live())
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
            let max_reaped = pool.size.current().saturating_sub(pool.options.min_size);

            // collect connections to reap
            let (reap, keep) = (0..max_reaped)
                // only connections waiting in the queue
                .filter_map(|_| pool.queue.try_pop(&pool.size))
                .partition::<Vec<_>, _>(|conn| {
                    is_beyond_idle(conn, &pool.options) || is_beyond_lifetime(conn, &pool.options)
                });

            for conn in keep {
                // return these connections to the pool first
                pool.queue.push(conn);
            }

            for conn in reap {
                let _ = conn.close().await;
            }

            sleep(period).await;
        }
    });
}
