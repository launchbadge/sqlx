use crate::{backend::Backend, query::RawQuery, row::FromSqlRow, error::Error};
use futures_core::{future::BoxFuture, stream::BoxStream};
use std::io;

pub trait Executor: Send {
    type Backend: Backend;

    fn execute<'c, 'q, Q: 'q + 'c>(&'c self, query: Q) -> BoxFuture<'c, Result<u64, Error>>
    where
        Q: RawQuery<'q, Backend = Self::Backend>;

    fn fetch<'c, 'q, T: 'c, Q: 'q + 'c>(&'c self, query: Q) -> BoxStream<'c, Result<T, Error>>
    where
        Q: RawQuery<'q, Backend = Self::Backend>,
        T: FromSqlRow<Self::Backend> + Send + Unpin;

    fn fetch_optional<'c, 'q, T: 'c, Q: 'q + 'c>(
        &'c self,
        query: Q,
    ) -> BoxFuture<'c, Result<Option<T>, Error>>
    where
        Q: RawQuery<'q, Backend = Self::Backend>,
        T: FromSqlRow<Self::Backend>;
}

impl<'e, E> Executor for &'e E
where
    E: Executor + Send + Sync,
{
    type Backend = E::Backend;

    #[inline]
    fn execute<'c, 'q, Q: 'q + 'c>(&'c self, query: Q) -> BoxFuture<'c, Result<u64, Error>>
    where
        Q: RawQuery<'q, Backend = Self::Backend>,
    {
        (*self).execute(query)
    }

    fn fetch<'c, 'q, T: 'c, Q: 'q + 'c>(&'c self, query: Q) -> BoxStream<'c, Result<T, Error>>
    where
        Q: RawQuery<'q, Backend = Self::Backend>,
        T: FromSqlRow<Self::Backend> + Send + Unpin,
    {
        (*self).fetch(query)
    }

    fn fetch_optional<'c, 'q, T: 'c, Q: 'q + 'c>(
        &'c self,
        query: Q,
    ) -> BoxFuture<'c, Result<Option<T>, Error>>
    where
        Q: RawQuery<'q, Backend = Self::Backend>,
        T: FromSqlRow<Self::Backend>,
    {
        (*self).fetch_optional(query)
    }
}
