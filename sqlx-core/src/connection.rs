use crate::{
    backend::Backend,
    describe::Describe,
    error::Error,
    executor::Executor,
    params::IntoQueryParameters,
    pool::{Live, SharedPool},
    row::{FromRow, Row},
};
use futures_core::{future::BoxFuture, stream::BoxStream};
use futures_util::stream::StreamExt;
use std::{
    ops::{Deref, DerefMut},
    sync::Arc,
    time::Instant,
};

pub struct Connection<DB>
where
    DB: Backend,
{
    live: Live<DB>,
}

impl<DB> Connection<DB>
where
    DB: Backend,
{
    pub(crate) fn new(live: Live<DB>) -> Self {
        Self { live }
    }

    pub async fn open(url: &str) -> crate::Result<Self> {
        Ok(Self::new(Live::unpooled(DB::open(url).await?)))
    }
}

impl<DB> Executor for Connection<DB>
where
    DB: Backend,
{
    type Backend = DB;

    fn execute<'e, 'q: 'e, I: 'e>(
        &'e mut self,
        query: &'q str,
        params: I,
    ) -> BoxFuture<'e, crate::Result<u64>>
    where
        I: IntoQueryParameters<Self::Backend> + Send,
    {
        self.live.execute(query, params)
    }

    fn fetch<'e, 'q: 'e, I: 'e, T: 'e>(
        &'e mut self,
        query: &'q str,
        params: I,
    ) -> BoxStream<'e, crate::Result<T>>
    where
        I: IntoQueryParameters<Self::Backend> + Send,
        T: FromRow<Self::Backend> + Send + Unpin,
    {
        self.live.fetch(query, params)
    }

    fn fetch_optional<'e, 'q: 'e, I: 'e, T: 'e>(
        &'e mut self,
        query: &'q str,
        params: I,
    ) -> BoxFuture<'e, crate::Result<Option<T>>>
    where
        I: IntoQueryParameters<Self::Backend> + Send,
        T: FromRow<Self::Backend> + Send,
    {
        self.live.fetch_optional(query, params)
    }

    fn describe<'e, 'q: 'e>(
        &'e mut self,
        query: &'q str,
    ) -> BoxFuture<'e, crate::Result<Describe<Self::Backend>>> {
        self.live.describe(query)
    }
}
