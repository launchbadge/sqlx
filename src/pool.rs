use crate::{
    backend::Backend, connection::RawConnection, executor::Executor, query::RawQuery, row::FromRow,
};
use crossbeam_queue::{ArrayQueue, SegQueue};
use futures_channel::oneshot;
use futures_core::{future::BoxFuture, stream::BoxStream};
use futures_util::stream::StreamExt;
use std::{
    io,
    ops::{Deref, DerefMut},
    sync::{
        atomic::{AtomicUsize, Ordering},
        Arc,
    },
    time::{Duration, Instant},
};

pub struct PoolOptions {
    pub max_size: usize,
    pub min_idle: Option<usize>,
    pub max_lifetime: Option<Duration>,
    pub idle_timeout: Option<Duration>,
    pub connection_timeout: Option<Duration>,
}

/// A database connection pool.
pub struct Pool<DB>(Arc<SharedPool<DB>>)
where
    DB: Backend;

impl<DB> Clone for Pool<DB>
where
    DB: Backend,
{
    fn clone(&self) -> Self {
        Self(Arc::clone(&self.0))
    }
}

impl<DB> Pool<DB>
where
    DB: Backend,
{
    // TODO: PoolBuilder
    pub fn new(url: &str, max_size: usize) -> Self {
        Self(Arc::new(SharedPool {
            url: url.to_owned(),
            idle: ArrayQueue::new(max_size),
            total: AtomicUsize::new(0),
            waiters: SegQueue::new(),
            options: PoolOptions {
                idle_timeout: None,
                connection_timeout: None,
                max_lifetime: None,
                max_size,
                min_idle: None,
            },
        }))
    }
}

struct SharedPool<DB>
where
    DB: Backend,
{
    url: String,
    idle: ArrayQueue<Idle<DB>>,
    waiters: SegQueue<oneshot::Sender<Live<DB>>>,
    total: AtomicUsize,
    options: PoolOptions,
}

impl<DB> SharedPool<DB>
where
    DB: Backend,
{
    async fn acquire(&self) -> io::Result<Live<DB>> {
        if let Ok(idle) = self.idle.pop() {
            return Ok(idle.live);
        }

        let total = self.total.load(Ordering::SeqCst);

        if total >= self.options.max_size {
            // Too many already, add a waiter and wait for
            // a free connection
            let (sender, reciever) = oneshot::channel();

            self.waiters.push(sender);

            // TODO: Handle errors here
            return Ok(reciever.await.unwrap());
        }

        self.total.store(total + 1, Ordering::SeqCst);

        let raw = <DB as Backend>::RawConnection::establish(&self.url).await?;

        let live = Live {
            raw,
            since: Instant::now(),
        };

        Ok(live)
    }

    fn release(&self, mut live: Live<DB>) {
        while let Ok(waiter) = self.waiters.pop() {
            live = match waiter.send(live) {
                Ok(()) => {
                    return;
                }

                Err(live) => live,
            };
        }

        let _ = self.idle.push(Idle {
            live,
            since: Instant::now(),
        });
    }
}

impl<DB> Executor for Pool<DB>
where
    DB: Backend,
{
    type Backend = DB;

    fn execute<'c, 'q, Q: 'q + 'c>(&'c self, query: Q) -> BoxFuture<'c, io::Result<u64>>
    where
        Q: RawQuery<'q, Backend = Self::Backend>,
    {
        Box::pin(async move {
            let live = self.0.acquire().await?;
            let mut conn = PooledConnection::new(&self.0, live);

            conn.execute(query).await
        })
    }

    fn fetch<'c, 'q, A: 'c, T: 'c, Q: 'q + 'c>(&'c self, query: Q) -> BoxStream<'c, io::Result<T>>
    where
        Q: RawQuery<'q, Backend = Self::Backend>,
        T: FromRow<A, Self::Backend> + Send + Unpin,
    {
        Box::pin(async_stream::try_stream! {
            let live = self.0.acquire().await?;
            let mut conn = PooledConnection::new(&self.0, live);
            let mut s = conn.fetch(query);

            while let Some(row) = s.next().await.transpose()? {
                yield T::from_row(row);
            }
        })
    }

    fn fetch_optional<'c, 'q, A: 'c, T: 'c, Q: 'q + 'c>(
        &'c self,
        query: Q,
    ) -> BoxFuture<'c, io::Result<Option<T>>>
    where
        Q: RawQuery<'q, Backend = Self::Backend>,
        T: FromRow<A, Self::Backend>,
    {
        Box::pin(async move {
            let live = self.0.acquire().await?;
            let mut conn = PooledConnection::new(&self.0, live);
            let row = conn.fetch_optional(query).await?;

            Ok(row.map(T::from_row))
        })
    }
}

struct PooledConnection<'a, DB>
where
    DB: Backend,
{
    shared: &'a Arc<SharedPool<DB>>,
    live: Option<Live<DB>>,
}

impl<'a, DB> PooledConnection<'a, DB>
where
    DB: Backend,
{
    fn new(shared: &'a Arc<SharedPool<DB>>, live: Live<DB>) -> Self {
        Self {
            shared,
            live: Some(live),
        }
    }
}

impl<DB> Deref for PooledConnection<'_, DB>
where
    DB: Backend,
{
    type Target = DB::RawConnection;

    fn deref(&self) -> &Self::Target {
        &self.live.as_ref().expect("connection use after drop").raw
    }
}

impl<DB> DerefMut for PooledConnection<'_, DB>
where
    DB: Backend,
{
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.live.as_mut().expect("connection use after drop").raw
    }
}

impl<DB> Drop for PooledConnection<'_, DB>
where
    DB: Backend,
{
    fn drop(&mut self) {
        if let Some(live) = self.live.take() {
            self.shared.release(live);
        }
    }
}

struct Idle<DB>
where
    DB: Backend,
{
    live: Live<DB>,
    since: Instant,
}

struct Live<DB>
where
    DB: Backend,
{
    raw: DB::RawConnection,
    since: Instant,
}
