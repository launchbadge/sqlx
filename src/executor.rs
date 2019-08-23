use crate::{backend::Backend, error::Error, query::QueryParameters, row::FromSqlRow};
use futures_core::{future::BoxFuture, stream::BoxStream};
use std::io;

pub trait Executor: Send {
    type Backend: Backend;

    fn execute<'c, 'q: 'c>(
        &'c self,
        query: &'q str,
        params: <Self::Backend as Backend>::QueryParameters,
    ) -> BoxFuture<'c, Result<u64, Error>>;

    fn fetch<'c, 'q: 'c, T: 'c>(
        &'c self,
        query: &'q str,
        params: <Self::Backend as Backend>::QueryParameters,
    ) -> BoxStream<'c, Result<T, Error>>
    where
        T: FromSqlRow<Self::Backend> + Send + Unpin;

    fn fetch_optional<'c, 'q: 'c, T: 'c>(
        &'c self,
        query: &'q str,
        params: <Self::Backend as Backend>::QueryParameters,
    ) -> BoxFuture<'c, Result<Option<T>, Error>>
    where
        T: FromSqlRow<Self::Backend>;
}

impl<'e, E> Executor for &'e E
where
    E: Executor + Send + Sync,
{
    type Backend = E::Backend;

    #[inline]
    fn execute<'c, 'q: 'c>(
        &'c self,
        query: &'q str,
        params: <Self::Backend as Backend>::QueryParameters,
    ) -> BoxFuture<'c, Result<u64, Error>> {
        (*self).execute(query, params)
    }

    fn fetch<'c, 'q: 'c, T: 'c>(
        &'c self,
        query: &'q str,
        params: <Self::Backend as Backend>::QueryParameters,
    ) -> BoxStream<'c, Result<T, Error>>
    where
        T: FromSqlRow<Self::Backend> + Send + Unpin,
    {
        (*self).fetch(query, params)
    }

    fn fetch_optional<'c, 'q: 'c, T: 'c>(
        &'c self,
        query: &'q str,
        params: <Self::Backend as Backend>::QueryParameters,
    ) -> BoxFuture<'c, Result<Option<T>, Error>>
    where
        T: FromSqlRow<Self::Backend>,
    {
        (*self).fetch_optional(query, params)
    }
}
