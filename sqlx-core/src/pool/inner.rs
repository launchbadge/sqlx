use std::{
    cmp,
    sync::{
        atomic::{AtomicBool, AtomicU32, Ordering},
        Arc,
    },
    time::Instant,
};

use async_std::{
    future::timeout,
    sync::{channel, Receiver, Sender},
    task,
};
use futures_util::future::FutureExt;

use crate::{error::Error, Connection, Database};

use super::{Idle, Options, Raw};

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
        // TODO: Establish [min_idle] connections

        let (pool_tx, pool_rx) = channel(options.max_size as usize);

        let pool = Arc::new(Self {
            url: url.to_owned(),
            pool_rx,
            size: AtomicU32::new(0),
            closed: AtomicBool::new(false),
            options,
        });

        conn_reaper(&pool, &pool_tx);

        Ok((pool, pool_tx))
    }

    pub fn options(&self) -> &Options {
        &self.options
    }

    pub(super) fn size(&self) -> u32 {
        self.size.load(Ordering::Acquire)
    }

    pub(super) fn num_idle(&self) -> usize {
        self.pool_rx.len()
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

        if let Some(raw) = self.try_acquire() {
            return Ok(raw);
        }

        while !self.closed.load(Ordering::Acquire) {
            let size = self.size.load(Ordering::Acquire);

            if size >= self.options.max_size {
                // Too many open connections
                // Wait until one is available

                // get the time between the deadline and now and use that as our timeout
                let max_wait = deadline
                    .checked_duration_since(Instant::now())
                    .ok_or(Error::PoolTimedOut)?;

                // don't sleep forever
                let mut idle = match timeout(max_wait, self.pool_rx.recv()).await {
                    Ok(Some(idle)) => idle,
                    Ok(None) => panic!("this isn't possible, we own a `pool_tx`"),
                    // try our acquire logic again
                    Err(_) => continue,
                };

                if self.closed.load(Ordering::Acquire) {
                    idle.close().await;
                    self.size.fetch_sub(1, Ordering::AcqRel);
                    return Err(Error::PoolClosed);
                }

                if should_reap(&idle, &self.options) {
                    // close the connection but don't really care about the result
                    idle.close().await;
                } else {
                    match idle.raw.inner.ping().await {
                        Ok(_) => return Ok(idle.raw),
                        // an error here means the other end has hung up or we lost connectivity
                        // either way we're fine to just discard the connection
                        // the error itself here isn't necessarily unexpected so WARN is too strong
                        Err(e) => log::info!("ping on idle connection returned error: {}", e),
                    }

                    // make sure the idle connection is gone explicitly before we open one
                    drop(idle);
                }

                // while we're still at max size, acquire a new connection
                return self.new_conn(deadline).await;
            }

            if self.size.compare_and_swap(size, size + 1, Ordering::AcqRel) == size {
                // Open a new connection and return directly
                return self.new_conn(deadline).await;
            }
        }

        Err(Error::PoolClosed)
    }

    async fn new_conn(&self, deadline: Instant) -> crate::Result<Raw<DB>> {
        while Instant::now() < deadline {
            if self.closed.load(Ordering::Acquire) {
                self.size.fetch_sub(1, Ordering::AcqRel);
                return Err(Error::PoolClosed);
            }

            // result here is `Result<Result<DB, Error>, TimeoutError>`
            match timeout(deadline - Instant::now(), DB::Connection::open(&self.url)).await {
                Ok(Ok(inner)) => {
                    return Ok(Raw {
                        inner,
                        created: Instant::now(),
                    })
                }
                // error while connecting, this should definitely be logged
                Ok(Err(e)) => log::warn!("error establishing a connection: {}", e),
                // timed out
                Err(_) => break,
            }
        }

        self.size.fetch_sub(1, Ordering::AcqRel);
        Err(Error::PoolTimedOut)
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

fn should_reap<DB: Database>(idle: &Idle<DB>, options: &Options) -> bool {
    // check if idle connection was within max lifetime (or not set)
    options.max_lifetime.map_or(true, |max| idle.raw.created.elapsed() < max)
        // and if connection wasn't idle too long (or not set)
        && options.idle_timeout.map_or(true, |timeout| idle.since.elapsed() < timeout)
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
                .saturating_sub(pool.options.min_idle);

            // collect connections to reap
            let (reap, keep) = (0..max_reaped)
                // only connections waiting in the queue
                .filter_map(|_| pool.pool_rx.recv().now_or_never()?)
                .partition::<Vec<_>, _>(|conn| should_reap(conn, &pool.options));

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
