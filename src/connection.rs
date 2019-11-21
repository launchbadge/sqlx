use crate::{
    backend::Backend,
    describe::Describe,
    error::Error,
    executor::Executor,
    pool::{Live, SharedPool},
    query::{IntoQueryParameters, QueryParameters},
    row::FromSqlRow,
};
use async_trait::async_trait;
use crossbeam_queue::SegQueue;
use crossbeam_utils::atomic::AtomicCell;
use futures_channel::oneshot::{channel, Sender};
use futures_core::{future::BoxFuture, stream::BoxStream};
use futures_util::stream::StreamExt;
use std::{
    sync::{
        atomic::{AtomicUsize, Ordering},
        Arc,
    },
    time::Instant,
};

/// A connection to the database.
///
/// This trait is not intended to be used directly. Instead [sqlx::Connection] or [sqlx::Pool] should be used instead, which provide
/// concurrent access and typed retrieval of results.
#[async_trait]
pub trait RawConnection: Send {
    // The database backend this type connects to.
    type Backend: Backend;

    /// Establish a new connection to the database server.
    async fn establish(url: &str) -> crate::Result<Self>
    where
        Self: Sized;

    /// Release resources for this database connection immediately.
    ///
    /// This method is not required to be called. A database server will eventually notice
    /// and clean up not fully closed connections.
    ///
    /// It is safe to close an already closed connection.
    async fn close(mut self) -> crate::Result<()>;

    /// Verifies a connection to the database is still alive.
    async fn ping(&mut self) -> crate::Result<()> {
        let _ = self
            .execute(
                "SELECT 1",
                <<Self::Backend as Backend>::QueryParameters>::new(),
            )
            .await?;

        Ok(())
    }

    async fn execute(
        &mut self,
        query: &str,
        params: <Self::Backend as Backend>::QueryParameters,
    ) -> crate::Result<u64>;

    fn fetch(
        &mut self,
        query: &str,
        params: <Self::Backend as Backend>::QueryParameters,
    ) -> BoxStream<'_, crate::Result<<Self::Backend as Backend>::Row>>;

    async fn fetch_optional(
        &mut self,
        query: &str,
        params: <Self::Backend as Backend>::QueryParameters,
    ) -> crate::Result<Option<<Self::Backend as Backend>::Row>>;

    async fn describe(&mut self, query: &str) -> crate::Result<Describe<Self::Backend>>;
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
    pub(crate) fn new(live: Live<DB>, pool: Option<Arc<SharedPool<DB>>>) -> Self {
        let shared = SharedConnection {
            live: AtomicCell::new(Some(live)),
            num_waiters: AtomicUsize::new(0),
            waiters: SegQueue::new(),
            pool,
        };

        Self(Arc::new(shared))
    }

    pub async fn establish(url: &str) -> crate::Result<Self> {
        let raw = <DB as Backend>::RawConnection::establish(url).await?;
        let live = Live {
            raw,
            since: Instant::now(),
        };

        Ok(Self::new(live, None))
    }

    /// Verifies a connection to the database is still alive.
    pub async fn ping(&self) -> crate::Result<()> {
        let mut live = self.0.acquire().await;
        live.raw.ping().await?;
        self.0.release(live);

        Ok(())
    }

    /// Analyze the SQL statement and report the inferred bind parameter types and returned
    /// columns.
    ///
    /// Mainly intended for use by sqlx-macros.
    pub async fn describe(&self, statement: &str) -> crate::Result<Describe<DB>> {
        let mut live = self.0.acquire().await;
        let ret = live.raw.describe(statement).await?;
        self.0.release(live);
        Ok(ret)
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
            let mut live = self.0.acquire().await;
            let result = live.raw.execute(query, params.into()).await;
            self.0.release(live);

            result
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
            let mut live = self.0.acquire().await;
            let mut s = live.raw.fetch(query, params.into());

            while let Some(row) = s.next().await.transpose()? {
                yield T::from_row(row);
            }

            drop(s);
            self.0.release(live);
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
            let mut live = self.0.acquire().await;
            let row = live.raw.fetch_optional(query, params.into()).await?;
            self.0.release(live);

            Ok(row.map(T::from_row))
        })
    }
}

struct SharedConnection<DB>
where
    DB: Backend,
{
    live: AtomicCell<Option<Live<DB>>>,
    pool: Option<Arc<SharedPool<DB>>>,
    num_waiters: AtomicUsize,
    waiters: SegQueue<Sender<Live<DB>>>,
}

impl<DB> SharedConnection<DB>
where
    DB: Backend,
{
    async fn acquire(&self) -> Live<DB> {
        if let Some(live) = self.live.swap(None) {
            // Fast path, this connection is not currently in use.
            // We can directly return the inner connection.
            return live;
        }

        let (sender, receiver) = channel();

        self.waiters.push(sender);
        self.num_waiters.fetch_add(1, Ordering::AcqRel);

        // Waiters are not dropped unless the pool is dropped
        // which would drop this future
        receiver
            .await
            .expect("waiter dropped without dropping connection")
    }

    fn release(&self, mut live: Live<DB>) {
        if self.num_waiters.load(Ordering::Acquire) > 0 {
            while let Ok(waiter) = self.waiters.pop() {
                self.num_waiters.fetch_sub(1, Ordering::AcqRel);

                live = match waiter.send(live) {
                    Ok(()) => {
                        return;
                    }

                    Err(live) => live,
                };
            }
        }

        self.live.store(Some(live));
    }
}

impl<DB> Drop for SharedConnection<DB>
where
    DB: Backend,
{
    fn drop(&mut self) {
        if let Some(pool) = &self.pool {
            // This error should not be able to happen
            pool.release(self.live.take().expect("drop while checked out"));
        }
    }
}
