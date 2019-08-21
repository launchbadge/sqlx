use crate::{
    backend::{Backend, BackendAssocRawQuery},
    executor::Executor,
    row::FromRow,
    serialize::ToSql,
    types::HasSqlType,
};
use futures_core::{future::BoxFuture, stream::BoxStream};
use std::io;

pub trait RawQuery<'q>: Sized + Send + Sync {
    type Backend: Backend;

    fn new(query: &'q str) -> Self;

    fn bind<T>(self, value: T) -> Self
    where
        Self::Backend: HasSqlType<T>,
        T: ToSql<Self::Backend>;

    fn finish(self, conn: &mut <Self::Backend as Backend>::RawConnection);
}

pub struct SqlQuery<'q, DB>
where
    DB: Backend,
{
    inner: <DB as BackendAssocRawQuery<'q, DB>>::RawQuery,
}

impl<'q, DB> SqlQuery<'q, DB>
where
    DB: Backend,
{
    #[inline]
    pub fn new(query: &'q str) -> Self {
        Self {
            inner: <DB as BackendAssocRawQuery<'q, DB>>::RawQuery::new(query),
        }
    }

    #[inline]
    pub fn bind<T>(mut self, value: T) -> Self
    where
        DB: HasSqlType<T>,
        T: ToSql<DB>,
    {
        self.inner = self.inner.bind(value);
        self
    }

    // TODO: These methods should go on a [Execute] trait (so more execut-able things can be defined)

    #[inline]
    pub fn execute<E>(self, executor: &'q E) -> BoxFuture<'q, io::Result<u64>>
    where
        E: Executor<Backend = DB>,
        <DB as BackendAssocRawQuery<'q, DB>>::RawQuery: 'q,
    {
        executor.execute(self.inner)
    }

    #[inline]
    pub fn fetch<E, T: 'q>(self, executor: &'q E) -> BoxStream<'q, io::Result<T>>
    where
        E: Executor<Backend = DB>,
        T: FromRow<DB> + Send + Unpin,
        <DB as BackendAssocRawQuery<'q, DB>>::RawQuery: 'q,
    {
        executor.fetch(self.inner)
    }

    #[inline]
    pub fn fetch_optional<E, T: 'q>(self, executor: &'q E) -> BoxFuture<'q, io::Result<Option<T>>>
    where
        E: Executor<Backend = DB>,
        T: FromRow<DB>,
        <DB as BackendAssocRawQuery<'q, DB>>::RawQuery: 'q,
    {
        executor.fetch_optional(self.inner)
    }
}

#[inline]
pub fn query<'q, DB>(query: &'q str) -> SqlQuery<'q, DB>
where
    DB: Backend,
{
    SqlQuery::new(query)
}
