use crate::{
    backend::Backend,
    connection::Connection,
    error::Error,
    executor::Executor,
    params::IntoQueryParameters,
    row::{FromRow, Row},
};
use futures_channel::oneshot;
use futures_core::{future::BoxFuture, stream::BoxStream};
use futures_util::future::{AbortHandle, AbortRegistration};
use futures_util::{
    future::{FutureExt, TryFutureExt},
    stream::StreamExt,
};
use std::{
    cmp,
    future::Future,
    marker::PhantomData,
    ops::{Deref, DerefMut},
    sync::{
        atomic::{AtomicBool, AtomicU32, AtomicUsize, Ordering},
        Arc,
    },
    time::{Duration, Instant},
};

use async_std::future::timeout;
use async_std::sync::{channel, Receiver, Sender};
use async_std::task;

use super::Options;

pub(crate) struct SharedPool<DB>
where
    DB: Backend,
{
    url: String,
    pool_rx: Receiver<Idle<DB>>,
    pool_tx: Sender<Idle<DB>>,
    size: AtomicU32,
    closed: AtomicBool,
    options: Options,
}

impl<DB> SharedPool<DB>
where
    DB: Backend,
{
    pub(crate) async fn new_arc(url: &str, options: Options) -> crate::Result<Arc<Self>> {
        // TODO: Establish [min_idle] connections

        let (pool_tx, pool_rx) = channel(options.max_size as usize);

        let pool = Arc::new(Self {
            url: url.to_owned(),
            pool_rx,
            pool_tx,
            size: AtomicU32::new(0),
            closed: AtomicBool::new(false),
            options,
        });

        conn_reaper(&pool);

        Ok(pool)
    }

    pub fn options(&self) -> &Options {
        &self.options
    }

    pub(crate) fn size(&self) -> u32 {
        self.size.load(Ordering::Acquire)
    }

    pub(crate) fn num_idle(&self) -> usize {
        self.pool_rx.len()
    }

    pub(crate) async fn close(&self) {
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
                Some(None) => panic!("we own a Sender how did this happen"),
                None => task::yield_now().await,
            }
        }
    }

    #[inline]
    pub(crate) fn try_acquire(&self) -> Option<Live<DB>> {
        if self.closed.load(Ordering::Acquire) {
            return None;
        }

        Some(self.pool_rx.recv().now_or_never()??.revive(&self.pool_tx))
    }

    pub(crate) async fn acquire(&self) -> crate::Result<Live<DB>> {
        let start = Instant::now();
        let deadline = start + self.options.connect_timeout;

        if let Some(live) = self.try_acquire() {
            return Ok(live);
        }

        while !self.closed.load(Ordering::Acquire) {
            let size = self.size.load(Ordering::Acquire);

            if size >= self.options.max_size {
                // Too many open connections
                // Wait until one is available

                // get the time between the deadline and now and use that as our timeout
                let max_wait = deadline
                    .checked_duration_since(Instant::now())
                    .ok_or(Error::TimedOut)?;

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
                        Ok(_) => return Ok(idle.revive(&self.pool_tx)),
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

    async fn new_conn(&self, deadline: Instant) -> crate::Result<Live<DB>> {
        while Instant::now() < deadline {
            if self.closed.load(Ordering::Acquire) {
                self.size.fetch_sub(1, Ordering::AcqRel);
                return Err(Error::PoolClosed);
            }

            // result here is `Result<Result<DB, Error>, TimeoutError>`
            match timeout(deadline - Instant::now(), DB::open(&self.url)).await {
                Ok(Ok(raw)) => return Ok(Live::pooled(raw, &self.pool_tx)),
                // error while connecting, this should definitely be logged
                Ok(Err(e)) => log::warn!("error establishing a connection: {}", e),
                // timed out
                Err(_) => break,
            }
        }

        self.size.fetch_sub(1, Ordering::AcqRel);
        Err(Error::TimedOut)
    }
}

struct Raw<DB> {
    pub(crate) inner: DB,
    pub(crate) created: Instant,
}

struct Idle<DB>
where
    DB: Backend,
{
    raw: Raw<DB>,
    #[allow(unused)]
    since: Instant,
}

impl<DB: Backend> Idle<DB> {
    fn revive(self, pool_tx: &Sender<Idle<DB>>) -> Live<DB> {
        Live {
            raw: Some(self.raw),
            pool_tx: Some(pool_tx.clone()),
        }
    }

    async fn close(self) {
        let _ = self.raw.inner.close().await;
    }
}

pub(crate) struct Live<DB>
where
    DB: Backend,
{
    raw: Option<Raw<DB>>,
    pool_tx: Option<Sender<Idle<DB>>>,
}

impl<DB: Backend> Live<DB> {
    pub fn unpooled(raw: DB) -> Self {
        Live {
            raw: Some(Raw {
                inner: raw,
                created: Instant::now(),
            }),
            pool_tx: None,
        }
    }

    fn pooled(raw: DB, pool_tx: &Sender<Idle<DB>>) -> Self {
        Live {
            raw: Some(Raw {
                inner: raw,
                created: Instant::now(),
            }),
            pool_tx: Some(pool_tx.clone()),
        }
    }

    fn release_mut(&mut self) {
        // `.release_mut()` will be called twice if `.release()` is called
        if let (Some(raw), Some(pool_tx)) = (self.raw.take(), self.pool_tx.as_ref()) {
            pool_tx
                .send(Idle {
                    raw,
                    since: Instant::now(),
                })
                .now_or_never()
                .expect("(bug) connection released into a full pool")
        }
    }
}

const DEREF_ERR: &str = "(bug) connection already released to pool";

impl<DB: Backend> Deref for Live<DB> {
    type Target = DB;

    fn deref(&self) -> &DB {
        &self.raw.as_ref().expect(DEREF_ERR).inner
    }
}

impl<DB: Backend> DerefMut for Live<DB> {
    fn deref_mut(&mut self) -> &mut DB {
        &mut self.raw.as_mut().expect(DEREF_ERR).inner
    }
}

impl<DB: Backend> Drop for Live<DB> {
    fn drop(&mut self) {
        self.release_mut()
    }
}

fn should_reap<DB: Backend>(idle: &Idle<DB>, options: &Options) -> bool {
    // check if idle connection was within max lifetime (or not set)
    options.max_lifetime.map_or(true, |max| idle.raw.created.elapsed() < max)
        // and if connection wasn't idle too long (or not set)
        && options.idle_timeout.map_or(true, |timeout| idle.since.elapsed() < timeout)
}

/// if `max_lifetime` or `idle_timeout` is set, spawn a task that reaps senescent connections
fn conn_reaper<DB: Backend>(pool: &Arc<SharedPool<DB>>) {
    if pool.options.max_lifetime.is_some() || pool.options.idle_timeout.is_some() {
        let pool = pool.clone();

        let reap_period = cmp::min(pool.options.max_lifetime, pool.options.idle_timeout)
            .expect("one of max_lifetime/idle_timeout should be `Some` at this point");

        task::spawn(async move {
            while !pool.closed.load(Ordering::AcqRel) {
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
                    pool.pool_tx.send(conn).await;
                }

                for conn in reap {
                    conn.close().await;
                    pool.size.fetch_sub(1, Ordering::AcqRel);
                }

                task::sleep(reap_period).await;
            }
        });
    }
}
