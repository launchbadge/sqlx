use crate::{backend::Backend, error::Error, query::IntoQueryParameters, row::FromSqlRow};
use futures_core::{future::BoxFuture, stream::BoxStream};
use futures_util::TryStreamExt;

pub trait Executor: Send {
    type Backend: Backend;

    fn execute<'c, 'q: 'c, A: 'c>(
        &'c self,
        query: &'q str,
        params: A,
    ) -> BoxFuture<'c, Result<u64, Error>>
    where
        A: IntoQueryParameters<Self::Backend> + Send;

    fn fetch<'c, 'q: 'c, T: 'c, A: 'c>(
        &'c self,
        query: &'q str,
        params: A,
    ) -> BoxStream<'c, Result<T, Error>>
    where
        A: IntoQueryParameters<Self::Backend> + Send,
        T: FromSqlRow<Self::Backend> + Send + Unpin;

    fn fetch_all<'c, 'q: 'c, T: 'c, A: 'c>(
        &'c self,
        query: &'q str,
        params: A,
    ) -> BoxFuture<'c, Result<Vec<T>, Error>>
    where
        A: IntoQueryParameters<Self::Backend> + Send,
        T: FromSqlRow<Self::Backend> + Send + Unpin
    {
        Box::pin(self.fetch(query, params).try_collect())
    }

    fn fetch_optional<'c, 'q: 'c, T: 'c, A: 'c>(
        &'c self,
        query: &'q str,
        params: A,
    ) -> BoxFuture<'c, Result<Option<T>, Error>>
    where
        A: IntoQueryParameters<Self::Backend> + Send,
        T: FromSqlRow<Self::Backend> + Send;

    fn fetch_one<'c, 'q: 'c, T: 'c, A: 'c>(
        &'c self,
        query: &'q str,
        params: A,
    ) -> BoxFuture<'c, Result<T, Error>>
    where
        A: IntoQueryParameters<Self::Backend> + Send,
        T: FromSqlRow<Self::Backend> + Send
    {
        let fut = self.fetch_optional(query, params);
        Box::pin(async move {
            fut.await?.ok_or(Error::NotFound)
        })
    }
}

impl<'e, E> Executor for &'e E
where
    E: Executor + Send + Sync,
{
    type Backend = E::Backend;

    #[inline]
    fn execute<'c, 'q: 'c, A: 'c>(
        &'c self,
        query: &'q str,
        params: A,
    ) -> BoxFuture<'c, Result<u64, Error>>
    where
        A: IntoQueryParameters<Self::Backend> + Send,
    {
        (*self).execute(query, params)
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
        (*self).fetch(query, params)
    }

    fn fetch_optional<'c, 'q: 'c, T: 'c, A: 'c>(
        &'c self,
        query: &'q str,
        params: A,
    ) -> BoxFuture<'c, Result<Option<T>, Error>>
    where
        A: IntoQueryParameters<Self::Backend> + Send,
        T: FromSqlRow<Self::Backend> + Send,
    {
        (*self).fetch_optional(query, params)
    }
}
