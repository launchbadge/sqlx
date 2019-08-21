use crate::{backend::Backend, query::RawQuery, row::FromRow};
use futures_core::{future::BoxFuture, stream::BoxStream};
use std::io;

pub trait Executor: Send {
    type Backend: Backend;

    fn execute<'c, 'q, Q: 'q + 'c>(&'c self, query: Q) -> BoxFuture<'c, io::Result<u64>>
    where
        Q: RawQuery<'q, Backend = Self::Backend>;

    fn fetch<'c, 'q, A: 'c, T: 'c, Q: 'q + 'c>(&'c self, query: Q) -> BoxStream<'c, io::Result<T>>
    where
        Q: RawQuery<'q, Backend = Self::Backend>,
        T: FromRow<A, Self::Backend> + Send + Unpin;

    fn fetch_optional<'c, 'q, A: 'c, T: 'c, Q: 'q + 'c>(
        &'c self,
        query: Q,
    ) -> BoxFuture<'c, io::Result<Option<T>>>
    where
        Q: RawQuery<'q, Backend = Self::Backend>,
        T: FromRow<A, Self::Backend>;
}

impl<'e, E> Executor for &'e E
where
    E: Executor + Send + Sync,
{
    type Backend = E::Backend;

    #[inline]
    fn execute<'c, 'q, Q: 'q + 'c>(&'c self, query: Q) -> BoxFuture<'c, io::Result<u64>>
    where
        Q: RawQuery<'q, Backend = Self::Backend>,
    {
        (*self).execute(query)
    }

    fn fetch<'c, 'q, A: 'c, T: 'c, Q: 'q + 'c>(&'c self, query: Q) -> BoxStream<'c, io::Result<T>>
    where
        Q: RawQuery<'q, Backend = Self::Backend>,
        T: FromRow<A, Self::Backend> + Send + Unpin,
    {
        (*self).fetch(query)
    }

    fn fetch_optional<'c, 'q, A: 'c, T: 'c, Q: 'q + 'c>(
        &'c self,
        query: Q,
    ) -> BoxFuture<'c, io::Result<Option<T>>>
    where
        Q: RawQuery<'q, Backend = Self::Backend>,
        T: FromRow<A, Self::Backend>,
    {
        (*self).fetch_optional(query)
    }
}
