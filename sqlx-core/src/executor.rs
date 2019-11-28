use crate::{
    backend::Backend,
    describe::Describe,
    error::Error,
    params::{IntoQueryParameters, QueryParameters},
    row::FromRow,
};
use futures_core::{future::BoxFuture, stream::BoxStream};
use futures_util::{TryFutureExt, TryStreamExt};

pub trait Executor: Send {
    type Backend: Backend;

    /// Verifies a connection to the database is still alive.
    fn ping<'e>(&'e mut self) -> BoxFuture<'e, crate::Result<()>> {
        Box::pin(
            self.execute(
                "SELECT 1",
                <Self::Backend as Backend>::QueryParameters::new(),
            )
            .map_ok(|_| ()),
        )
    }

    fn execute<'e, 'q: 'e, I: 'e>(
        &'e mut self,
        query: &'q str,
        params: I,
    ) -> BoxFuture<'e, crate::Result<u64>>
    where
        I: IntoQueryParameters<Self::Backend> + Send;

    fn fetch<'e, 'q: 'e, I: 'e, O: 'e, T: 'e>(
        &'e mut self,
        query: &'q str,
        params: I,
    ) -> BoxStream<'e, crate::Result<T>>
    where
        I: IntoQueryParameters<Self::Backend> + Send,
        T: FromRow<Self::Backend, O> + Send + Unpin;

    fn fetch_all<'e, 'q: 'e, I: 'e, O: 'e, T: 'e>(
        &'e mut self,
        query: &'q str,
        params: I,
    ) -> BoxFuture<'e, crate::Result<Vec<T>>>
    where
        I: IntoQueryParameters<Self::Backend> + Send,
        T: FromRow<Self::Backend, O> + Send + Unpin,
    {
        Box::pin(self.fetch(query, params).try_collect())
    }

    fn fetch_optional<'e, 'q: 'e, I: 'e, O: 'e, T: 'e>(
        &'e mut self,
        query: &'q str,
        params: I,
    ) -> BoxFuture<'e, crate::Result<Option<T>>>
    where
        I: IntoQueryParameters<Self::Backend> + Send,
        T: FromRow<Self::Backend, O> + Send;

    fn fetch_one<'e, 'q: 'e, I: 'e, O: 'e, T: 'e>(
        &'e mut self,
        query: &'q str,
        params: I,
    ) -> BoxFuture<'e, crate::Result<T>>
    where
        I: IntoQueryParameters<Self::Backend> + Send,
        T: FromRow<Self::Backend, O> + Send,
    {
        let fut = self.fetch_optional(query, params);
        Box::pin(async move { fut.await?.ok_or(Error::NotFound) })
    }

    /// Analyze the SQL statement and report the inferred bind parameter types and returned
    /// columns.
    fn describe<'e, 'q: 'e>(
        &'e mut self,
        query: &'q str,
    ) -> BoxFuture<'e, crate::Result<Describe<Self::Backend>>>;
}
