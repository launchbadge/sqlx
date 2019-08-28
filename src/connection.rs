use crate::{
    backend::Backend,
    error::Error,
    executor::Executor,
    query::{IntoQueryParameters, QueryParameters},
    row::FromSqlRow,
};
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

/// A connection to the database.
///
/// This trait is not intended to be used directly. Instead [sqlx::Connection] or [sqlx::Pool] should be used instead, which provide
/// concurrent access and typed retrieval of results.
pub trait RawConnection: Send {
    /// The database backend this type connects to.
    type Backend: Backend;

    /// Establish a new connection to the database server.
    fn establish(url: &str) -> BoxFuture<Result<Self, Error>>
    where
        Self: Sized;

    /// Release resources for this database connection immediately.
    ///
    /// This method is not required to be called. A database server will eventually notice
    /// and clean up not fully closed connections.
    fn finalize<'c>(&'c mut self) -> BoxFuture<'c, Result<(), Error>>;

    fn execute<'c>(
        &'c mut self,
        query: &str,
        params: <Self::Backend as Backend>::QueryParameters,
    ) -> BoxFuture<'c, Result<u64, Error>>;

    fn fetch<'c>(
        &'c mut self,
        query: &str,
        params: <Self::Backend as Backend>::QueryParameters,
    ) -> BoxStream<'c, Result<<Self::Backend as Backend>::Row, Error>>;

    fn fetch_optional<'c>(
        &'c mut self,
        query: &str,
        params: <Self::Backend as Backend>::QueryParameters,
    ) -> BoxFuture<'c, Result<Option<<Self::Backend as Backend>::Row>, Error>>;
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
    pub async fn establish(url: &str) -> Result<Self, Error> {
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

    fn execute<'c, 'q: 'c, A: 'c>(
        &'c self,
        query: &'q str,
        params: A,
    ) -> BoxFuture<'c, Result<u64, Error>>
    where
        A: IntoQueryParameters<Self::Backend> + Send,
    {
        Box::pin(async move {
            let mut conn = self.get().await;
            conn.execute(query, params.into()).await
        })
    }

    fn fetch<'c, 'q: 'c, T: 'c, A: 'c>(
        &'c self,
        query: &'q str,
        params: A,
    ) -> BoxStream<'c, Result<T, Error>>
    where
        A: IntoQueryParameters<Self::Backend> + Send,
        T: FromSqlRow<Self::Backend> + Send + Unpin,
    {
        Box::pin(async_stream::try_stream! {
            let mut conn = self.get().await;
            let mut s = conn.fetch(query, params.into());

            while let Some(row) = s.next().await.transpose()? {
                yield T::from_row(row);
            }
        })
    }

    fn fetch_optional<'c, 'q: 'c, T: 'c, A: 'c>(
        &'c self,
        query: &'q str,
        params: A,
    ) -> BoxFuture<'c, Result<Option<T>, Error>>
    where
        A: IntoQueryParameters<Self::Backend> + Send,
        T: FromSqlRow<Self::Backend>,
    {
        Box::pin(async move {
            let mut conn = self.get().await;
            let row = conn.fetch_optional(query, params.into()).await?;

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
