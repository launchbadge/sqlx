use crate::{backend::Backend, error::Error, params::IntoQueryParameters, row::FromRow};
use futures_core::{future::BoxFuture, stream::BoxStream};
use futures_util::TryStreamExt;

pub trait Executor: Send {
    type Backend: Backend;

    fn execute<'c, 'q: 'c, I: 'c>(
        &'c mut self,
        query: &'q str,
        params: I,
    ) -> BoxFuture<'c, Result<u64, Error>>
    where
        I: IntoQueryParameters<Self::Backend> + Send;

    fn fetch<'c, 'q: 'c, I: 'c, O: 'c, T: 'c>(
        &'c mut self,
        query: &'q str,
        params: I,
    ) -> BoxStream<'c, Result<T, Error>>
    where
        I: IntoQueryParameters<Self::Backend> + Send,
        T: FromRow<Self::Backend, O> + Send + Unpin;

    fn fetch_all<'c, 'q: 'c, I: 'c, O: 'c, T: 'c>(
        &'c mut self,
        query: &'q str,
        params: I,
    ) -> BoxFuture<'c, Result<Vec<T>, Error>>
    where
        I: IntoQueryParameters<Self::Backend> + Send,
        T: FromRow<Self::Backend, O> + Send + Unpin,
    {
        Box::pin(self.fetch(query, params).try_collect())
    }

    fn fetch_optional<'c, 'q: 'c, I: 'c, O: 'c, T: 'c>(
        &'c mut self,
        query: &'q str,
        params: I,
    ) -> BoxFuture<'c, Result<Option<T>, Error>>
    where
        I: IntoQueryParameters<Self::Backend> + Send,
        T: FromRow<Self::Backend, O> + Send;

    fn fetch_one<'c, 'q: 'c, I: 'c, O: 'c, T: 'c>(
        &'c mut self,
        query: &'q str,
        params: I,
    ) -> BoxFuture<'c, Result<T, Error>>
    where
        I: IntoQueryParameters<Self::Backend> + Send,
        T: FromRow<Self::Backend, O> + Send,
    {
        let fut = self.fetch_optional(query, params);
        Box::pin(async move { fut.await?.ok_or(Error::NotFound) })
    }
}
