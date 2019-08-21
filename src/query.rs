use crate::{
    backend::{Backend, BackendAssocRawQuery},
    executor::Executor,
    pool::Pool,
    row::FromRow,
    serialize::ToSql,
    types::{AsSqlType, HasSqlType},
};
use futures::{future::BoxFuture, stream::BoxStream};
use std::io;

pub trait RawQuery<'q>: Sized + Send + Sync {
    type Backend: Backend;

    fn new(query: &'q str) -> Self;

    fn bind_as<ST, T>(self, value: T) -> Self
    where
        Self::Backend: HasSqlType<ST>,
        T: ToSql<ST, Self::Backend>;

    fn finish(self, conn: &mut <Self::Backend as Backend>::RawConnection);
}

pub struct SqlQuery<'q, E>
where
    E: Executor,
{
    inner:
        <<E as Executor>::Backend as BackendAssocRawQuery<'q, <E as Executor>::Backend>>::RawQuery,
}

impl<'q, E> SqlQuery<'q, E>
where
    E: Executor,
{
    #[inline]
    pub fn new(query: &'q str) -> Self {
        Self {
            inner: <<E as Executor>::Backend as BackendAssocRawQuery<
                'q,
                <E as Executor>::Backend,
            >>::RawQuery::new(query),
        }
    }

    #[inline]
    pub fn bind<T>(self, value: T) -> Self
    where
        E::Backend: HasSqlType<<T as AsSqlType<E::Backend>>::SqlType>,
        T: AsSqlType<E::Backend> + ToSql<<T as AsSqlType<E::Backend>>::SqlType, E::Backend>,
    {
        self.bind_as::<T::SqlType, T>(value)
    }

    #[inline]
    pub fn bind_as<ST, T>(mut self, value: T) -> Self
    where
        E::Backend: HasSqlType<ST>,
        T: ToSql<ST, E::Backend>,
    {
        self.inner = self.inner.bind_as::<ST, T>(value);
        self
    }

    // TODO: These methods should go on a [Execute] trait (so more execut-able things can be defined)

    #[inline]
    pub fn execute(self, executor: &'q E) -> BoxFuture<'q, io::Result<u64>> {
        executor.execute(self.inner)
    }

    #[inline]
    pub fn fetch<A: 'q, T: 'q>(self, executor: &'q E) -> BoxStream<'q, io::Result<T>>
    where
        T: FromRow<A, E::Backend> + Send + Unpin,
    {
        executor.fetch(self.inner)
    }

    #[inline]
    pub fn fetch_optional<A: 'q, T: 'q>(
        self,
        executor: &'q E,
    ) -> BoxFuture<'q, io::Result<Option<T>>>
    where
        T: FromRow<A, E::Backend>,
    {
        executor.fetch_optional(self.inner)
    }
}

#[inline]
pub fn query<'q, E>(query: &'q str) -> SqlQuery<'q, E>
where
    E: Executor,
{
    SqlQuery::new(query)
}
