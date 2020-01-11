use std::cmp;
use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};
use std::sync::Arc;
use std::time::Instant;

use async_std::prelude::FutureExt as _;
use async_std::sync::{channel, Receiver, Sender};
use async_std::task;
use futures_util::future::FutureExt as _;

use super::{Idle, Options, Raw};
use crate::{error::Error, Connection, Database};

pub(super) struct SharedPool<DB>
where
    DB: Database,
{
    url: String,
    pool_rx: Receiver<Idle<DB>>,
    size: AtomicU32,
    closed: AtomicBool,
    options: Options,
}

impl<DB> SharedPool<DB>
where
    DB: Database,
    DB::Connection: Connection<Database = DB>,
{
    pub(super) async fn new_arc(
        url: &str,
        options: Options,
    ) -> crate::Result<(Arc<Self>, Sender<Idle<DB>>)> {
        let (pool_tx, pool_rx) = channel(options.max_size as usize);

        let pool = Arc::new(Self {
            url: url.to_owned(),
            pool_rx,
            size: AtomicU32::new(0),
            closed: AtomicBool::new(false),
            options,
        });

        for _ in 0..pool.options.min_size {
            let raw = pool
                .eventually_connect(Instant::now() + pool.options.connect_timeout)
                .await?;

            pool_tx
                .send(Idle {
                    raw,
                    since: Instant::now(),
                })
                .await;
        }

        conn_reaper(&pool, &pool_tx);

        Ok((pool, pool_tx))
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
        self.pool_rx.len()
    }

    pub(super) fn closed(&self) -> bool {
        self.closed.load(Ordering::SeqCst)
    }

    pub(super) async fn close(&self) {
        self.closed.store(true, Ordering::Release);

        while self.size.load(Ordering::Acquire) > 0 {
            // don't block on the receiver because we own one Sender so it should never return
            // `None`; a `select!()` would also work but that produces more complicated code
            // and a timeout isn't necessarily appropriate
            match self.pool_rx.recv().now_or_never() {
                Some(Some(idle)) => {
                    idle.close().await;
                    self.size.fetch_sub(1, Ordering::AcqRel);
                }
                Some(None) => {
                    log::warn!("was not able to close all connections");
                    break;
                }
                None => task::yield_now().await,
            }
        }
    }

    #[inline]
    pub(super) fn try_acquire(&self) -> Option<Raw<DB>> {
        if self.closed.load(Ordering::Acquire) {
            return None;
        }

        Some(self.pool_rx.recv().now_or_never()??.raw)
    }

    pub(super) async fn acquire(&self) -> crate::Result<Raw<DB>> {
        let start = Instant::now();
        let deadline = start + self.options.connect_timeout;

        // Unless the pool has been closed ...
        while !self.closed.load(Ordering::Acquire) {
            let size = self.size.load(Ordering::Acquire);

            // Attempt to immediately acquire a connection. This will return Some
            // if there is an idle connection in our channel.
            let mut idle = if let Some(idle) = self.pool_rx.recv().now_or_never() {
                let idle = match idle {
                    Some(idle) => idle,

                    // This isn't possible. [Pool] owns the sender and [SharedPool]
                    // owns the receiver.
                    None => unreachable!(),
                };

                idle
            } else if size >= self.options.max_size {
                // Too many open connections
                // Wait until one is available

                // get the time between the deadline and now and use that as our timeout
                let until = deadline
                    .checked_duration_since(Instant::now())
                    .ok_or(Error::PoolTimedOut(None))?;

                // don't sleep forever
                let idle = match self.pool_rx.recv().timeout(until).await {
                    // A connection was returned to the pool
                    Ok(Some(idle)) => idle,

                    // This isn't possible. [Pool] owns the sender and [SharedPool]
                    // owns the receiver.
                    Ok(None) => unreachable!(),

                    // Timed out waiting for a connection
                    // Error is not forwarded as its useless context
                    Err(_) => {
                        return Err(Error::PoolTimedOut(None));
                    }
                };

                idle
            } else if self.size.compare_and_swap(size, size + 1, Ordering::AcqRel) == size {
                // pool has slots available; open a new connection
                match self.connect(deadline).await {
                    Ok(Some(conn)) => return Ok(conn),
                    // [size] is internally decremented on _retry_ and _error_
                    Ok(None) => continue,
                    Err(e) => return Err(e),
                }
            } else {
                continue;
            };

            // If pool was closed while waiting for a connection,
            // release the connection
            if self.closed.load(Ordering::Acquire) {
                idle.close().await;
                self.size.fetch_sub(1, Ordering::AcqRel);

                return Err(Error::PoolClosed);
            }

            // If the connection we pulled has expired, close the connection and
            // immediately create a new connection
            if is_beyond_lifetime(&idle.raw, &self.options) {
                // close the connection but don't really care about the result
                let _ = idle.close().await;
            } else if self.options.test_on_acquire {
                // TODO: Check on acquire should be a configuration setting
                // Check that the connection is still live
                match idle.raw.inner.ping().await {
                    // Connection still seems to respond
                    Ok(_) => return Ok(idle.raw),

                    // an error here means the other end has hung up or we lost connectivity
                    // either way we're fine to just discard the connection
                    // the error itself here isn't necessarily unexpected so WARN is too strong
                    Err(e) => log::info!("ping on idle connection returned error: {}", e),
                }

                // make sure the idle connection is gone explicitly before we open one
                // this will close the resources for the stream on our side
                drop(idle);
            } else {
                // No need to re-connect
                return Ok(idle.raw);
            }

            // while there is still room in the pool, acquire a new connection
            match self.connect(deadline).await {
                Ok(Some(conn)) => return Ok(conn),
                // [size] is internally decremented on _retry_ and _error_
                Ok(None) => continue,
                Err(e) => return Err(e),
            }
        }

        Err(Error::PoolClosed)
    }

    async fn eventually_connect(&self, deadline: Instant) -> crate::Result<Raw<DB>> {
        loop {
            // [connect] will raise an error when past deadline
            // [connect] returns None if its okay to retry
            if let Some(conn) = self.connect(deadline).await? {
                return Ok(conn);
            }
        }
    }

    async fn connect(&self, deadline: Instant) -> crate::Result<Option<Raw<DB>>> {
        // FIXME: Code between `-` is duplicate with [acquire]
        // ---------------------------------

        // get the time between the deadline and now and use that as our timeout
        let until = deadline
            .checked_duration_since(Instant::now())
            .ok_or(Error::PoolTimedOut(None))?;

        // If pool was closed while waiting for a connection,
        // release the connection
        if self.closed.load(Ordering::Acquire) {
            self.size.fetch_sub(1, Ordering::AcqRel); // ?

            return Err(Error::PoolClosed);
        }

        // ---------------------------------

        // result here is `Result<Result<DB, Error>, TimeoutError>`
        match DB::Connection::open(&self.url).timeout(until).await {
            // successfully established connection
            Ok(Ok(inner)) => {
                Ok(Some(Raw {
                    inner,
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

impl<DB: Database> Idle<DB>
where
    DB::Connection: Connection<Database = DB>,
{
    async fn close(self) {
        let _ = self.raw.inner.close().await;
    }
}

// NOTE: Function names here are bizzare. Helpful help would be appreciated.

fn is_beyond_lifetime<DB: Database>(raw: &Raw<DB>, options: &Options) -> bool {
    // check if connection was within max lifetime (or not set)
    options
        .max_lifetime
        .map_or(false, |max| raw.created.elapsed() > max)
}

fn is_beyond_idle<DB: Database>(idle: &Idle<DB>, options: &Options) -> bool {
    // if connection wasn't idle too long (or not set)
    options
        .idle_timeout
        .map_or(false, |timeout| idle.since.elapsed() > timeout)
}

/// if `max_lifetime` or `idle_timeout` is set, spawn a task that reaps senescent connections
fn conn_reaper<DB: Database>(pool: &Arc<SharedPool<DB>>, pool_tx: &Sender<Idle<DB>>)
where
    DB::Connection: Connection<Database = DB>,
{
    let period = match (pool.options.max_lifetime, pool.options.idle_timeout) {
        (Some(it), None) | (None, Some(it)) => it,

        (Some(a), Some(b)) => cmp::min(a, b),

        (None, None) => return,
    };

    let pool = pool.clone();
    let pool_tx = pool_tx.clone();

    task::spawn(async move {
        while !pool.closed.load(Ordering::Acquire) {
            // reap at most the current size minus the minimum idle
            let max_reaped = pool
                .size
                .load(Ordering::Acquire)
                .saturating_sub(pool.options.min_size);

            // collect connections to reap
            let (reap, keep) = (0..max_reaped)
                // only connections waiting in the queue
                .filter_map(|_| pool.pool_rx.recv().now_or_never()?)
                .partition::<Vec<_>, _>(|conn| {
                    is_beyond_idle(conn, &pool.options)
                        || is_beyond_lifetime(&conn.raw, &pool.options)
                });

            for conn in keep {
                // return these connections to the pool first
                pool_tx.send(conn).await;
            }

            for conn in reap {
                conn.close().await;
                pool.size.fetch_sub(1, Ordering::AcqRel);
            }

            task::sleep(period).await;
        }
    });
}
