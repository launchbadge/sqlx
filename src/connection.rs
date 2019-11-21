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
pub trait RawConnection: Send + Sync {
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

pub struct Connection<DB>
where
    DB: Backend,
{
    live: Option<Live<DB>>,
    pool: Option<Arc<SharedPool<DB>>>,
}

impl<DB> Connection<DB>
where
    DB: Backend,
{
    pub(crate) fn new(live: Live<DB>, pool: Option<Arc<SharedPool<DB>>>) -> Self {
        Self {
            live: Some(live),
            pool,
        }
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
    pub async fn ping(&mut self) -> crate::Result<()> {
        self.live.as_mut().expect("released").raw.ping().await
    }

    /// Analyze the SQL statement and report the inferred bind parameter types and returned
    /// columns.
    ///
    /// Mainly intended for use by sqlx-macros.
    pub async fn describe(&mut self, statement: &str) -> crate::Result<Describe<DB>> {
        self.live
            .as_mut()
            .expect("released")
            .raw
            .describe(statement)
            .await
    }
}

impl<DB> Executor for Connection<DB>
where
    DB: Backend,
{
    type Backend = DB;

    fn execute<'c, 'q: 'c, A: 'c>(
        &'c mut self,
        query: &'q str,
        params: A,
    ) -> BoxFuture<'c, Result<u64, Error>>
    where
        A: IntoQueryParameters<Self::Backend> + Send,
    {
        Box::pin(async move {
            self.live
                .as_mut()
                .expect("released")
                .raw
                .execute(query, params.into())
                .await
        })
    }

    fn fetch<'c, 'q: 'c, T: 'c, A: 'c>(
        &'c mut self,
        query: &'q str,
        params: A,
    ) -> BoxStream<'c, Result<T, Error>>
    where
        A: IntoQueryParameters<Self::Backend> + Send,
        T: FromSqlRow<Self::Backend> + Send + Unpin,
    {
        Box::pin(async_stream::try_stream! {
            let mut s = self.live.as_mut().expect("released").raw.fetch(query, params.into());

            while let Some(row) = s.next().await.transpose()? {
                yield T::from_row(row);
            }
        })
    }

    fn fetch_optional<'c, 'q: 'c, T: 'c, A: 'c>(
        &'c mut self,
        query: &'q str,
        params: A,
    ) -> BoxFuture<'c, Result<Option<T>, Error>>
    where
        A: IntoQueryParameters<Self::Backend> + Send,
        T: FromSqlRow<Self::Backend>,
    {
        Box::pin(async move {
            let row = self
                .live
                .as_mut()
                .expect("released")
                .raw
                .fetch_optional(query, params.into())
                .await?;

            Ok(row.map(T::from_row))
        })
    }
}

impl<DB> Drop for Connection<DB>
where
    DB: Backend,
{
    fn drop(&mut self) {
        if let Some(pool) = &self.pool {
            if let Some(live) = self.live.take() {
                pool.release(live);
            }
        }
    }
}
