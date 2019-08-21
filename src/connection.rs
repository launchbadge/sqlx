use crate::{backend::Backend, executor::Executor, query::RawQuery, row::FromRow};
use crossbeam_queue::SegQueue;
use crossbeam_utils::atomic::AtomicCell;
use futures_channel::oneshot::{channel, Sender};
use futures_core::{future::BoxFuture, stream::BoxStream};
use futures_util::stream::StreamExt;
use std::{
    io,
    ops::{Deref, DerefMut},
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
};

pub trait RawConnection: Send {
    type Backend: Backend;

    /// Establish a new connection to the database server.
    fn establish(url: &str) -> BoxFuture<io::Result<Self>>
    where
        Self: Sized;

    /// Release resources for this database connection immediately.
    /// This method is not required to be called. A database server will eventually notice
    /// and clean up not fully closed connections.
    fn finalize<'c>(&'c mut self) -> BoxFuture<'c, io::Result<()>>;

    fn execute<'c, 'q, Q: 'q>(&'c mut self, query: Q) -> BoxFuture<'c, io::Result<u64>>
    where
        Q: RawQuery<'q, Backend = Self::Backend>;

    fn fetch<'c, 'q, Q: 'q>(
        &'c mut self,
        query: Q,
    ) -> BoxStream<'c, io::Result<<Self::Backend as Backend>::Row>>
    where
        Q: RawQuery<'q, Backend = Self::Backend>;

    fn fetch_optional<'c, 'q, Q: 'q>(
        &'c mut self,
        query: Q,
    ) -> BoxFuture<'c, io::Result<Option<<Self::Backend as Backend>::Row>>>
    where
        Q: RawQuery<'q, Backend = Self::Backend>;
}

pub struct Connection<DB>(Arc<SharedConnection<DB>>)
where
    DB: Backend;

impl<DB> Clone for Connection<DB>
where
    DB: Backend,
{
    #[inline]
    fn clone(&self) -> Self {
        Self(Arc::clone(&self.0))
    }
}

impl<DB> Connection<DB>
where
    DB: Backend,
{
    pub async fn establish(url: &str) -> io::Result<Self> {
        let raw = <DB as Backend>::RawConnection::establish(url).await?;
        let shared = SharedConnection {
            raw: AtomicCell::new(Some(Box::new(raw))),
            waiting: AtomicBool::new(false),
            waiters: SegQueue::new(),
        };

        Ok(Self(Arc::new(shared)))
    }

    async fn get(&self) -> ConnectionFairy<'_, DB> {
        ConnectionFairy::new(&self.0, self.0.acquire().await)
    }
}

impl<DB> Executor for Connection<DB>
where
    DB: Backend,
{
    type Backend = DB;

    fn execute<'c, 'q, Q: 'q + 'c>(&'c self, query: Q) -> BoxFuture<'c, io::Result<u64>>
    where
        Q: RawQuery<'q, Backend = Self::Backend>,
    {
        Box::pin(async move {
            let mut conn = self.get().await;
            conn.execute(query).await
        })
    }

    fn fetch<'c, 'q, A: 'c, T: 'c, Q: 'q + 'c>(&'c self, query: Q) -> BoxStream<'c, io::Result<T>>
    where
        Q: RawQuery<'q, Backend = Self::Backend>,
        T: FromRow<A, Self::Backend> + Send + Unpin,
    {
        Box::pin(async_stream::try_stream! {
            let mut conn = self.get().await;
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
            let mut conn = self.get().await;
            let row = conn.fetch_optional(query).await?;

            Ok(row.map(T::from_row))
        })
    }
}

struct SharedConnection<DB>
where
    DB: Backend,
{
    raw: AtomicCell<Option<Box<DB::RawConnection>>>,
    waiting: AtomicBool,
    waiters: SegQueue<Sender<Box<DB::RawConnection>>>,
}

impl<DB> SharedConnection<DB>
where
    DB: Backend,
{
    async fn acquire(&self) -> Box<DB::RawConnection> {
        if let Some(raw) = self.raw.swap(None) {
            // Fast path, this connection is not currently in use.
            // We can directly return the inner connection.
            return raw;
        }

        let (sender, receiver) = channel();

        self.waiters.push(sender);
        self.waiting.store(true, Ordering::Release);

        // TODO: Handle this error
        receiver.await.unwrap()
    }

    fn release(&self, mut raw: Box<DB::RawConnection>) {
        // If we have any waiters, iterate until we find a non-dropped waiter
        if self.waiting.load(Ordering::Acquire) {
            while let Ok(waiter) = self.waiters.pop() {
                raw = match waiter.send(raw) {
                    Err(raw) => raw,
                    Ok(_) => {
                        return;
                    }
                };
            }
        }

        // Otherwise, just re-store the connection until
        // we are needed again
        self.raw.store(Some(raw));
    }
}

struct ConnectionFairy<'a, DB>
where
    DB: Backend,
{
    shared: &'a Arc<SharedConnection<DB>>,
    raw: Option<Box<DB::RawConnection>>,
}

impl<'a, DB> ConnectionFairy<'a, DB>
where
    DB: Backend,
{
    #[inline]
    fn new(shared: &'a Arc<SharedConnection<DB>>, raw: Box<DB::RawConnection>) -> Self {
        Self {
            shared,
            raw: Some(raw),
        }
    }
}

impl<DB> Deref for ConnectionFairy<'_, DB>
where
    DB: Backend,
{
    type Target = DB::RawConnection;

    #[inline]
    fn deref(&self) -> &Self::Target {
        self.raw.as_ref().expect("connection use after drop")
    }
}

impl<DB> DerefMut for ConnectionFairy<'_, DB>
where
    DB: Backend,
{
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.raw.as_mut().expect("connection use after drop")
    }
}

impl<DB> Drop for ConnectionFairy<'_, DB>
where
    DB: Backend,
{
    fn drop(&mut self) {
        if let Some(raw) = self.raw.take() {
            self.shared.release(raw);
        }
    }
}
