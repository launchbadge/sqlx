use crate::{
    backend::Backend,
    describe::Describe,
    error::Error,
    executor::Executor,
    pool::{Live, SharedPool},
    query::IntoQueryParameters,
    row::FromSqlRow,
};
use futures_core::{future::BoxFuture, stream::BoxStream};
use futures_util::stream::StreamExt;
use std::{sync::Arc, time::Instant};

pub struct Connection<DB>
where
    DB: Backend,
{
    live: Live<DB>,
    pool: Option<Arc<SharedPool<DB>>>,
}

impl<DB> Connection<DB>
where
    DB: Backend,
{
    pub(crate) fn new(live: Live<DB>, pool: Option<Arc<SharedPool<DB>>>) -> Self {
        Self { live, pool }
    }

    pub async fn open(url: &str) -> crate::Result<Self> {
        let raw = DB::open(url).await?;
        Ok(Self::new(Live::unpooled(raw), None))
    }

    /// Verifies a connection to the database is still alive.
    pub async fn ping(&mut self) -> crate::Result<()> {
        self.live.ping().await
    }

    /// Analyze the SQL statement and report the inferred bind parameter types and returned
    /// columns.
    ///
    /// Mainly intended for use by sqlx-macros.
    pub async fn describe(&mut self, statement: &str) -> crate::Result<Describe<DB>> {
        self.live.describe(statement).await
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
        Box::pin(async move { self.live.execute(query, params.into_params()).await })
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
            let mut s = self.live.fetch(query, params.into_params());

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
                .fetch_optional(query, params.into_params())
                .await?;

            Ok(row.map(T::from_row))
        })
    }
}
