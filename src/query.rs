use crate::{
    backend::Backend,
    executor::Executor,
    pool::Pool,
    row::FromRow,
    serialize::ToSql,
    types::{AsSqlType, HasSqlType},
};
use futures::{future::BoxFuture, stream::BoxStream};
use std::io;

pub trait Query<'q>: Sized + Send + Sync {
    type Backend: Backend;

    fn new(query: &'q str) -> Self;

    #[inline]
    fn bind<T>(self, value: T) -> Self
    where
        Self::Backend: HasSqlType<<T as AsSqlType<Self::Backend>>::SqlType>,
        T: AsSqlType<Self::Backend>
            + ToSql<<T as AsSqlType<Self::Backend>>::SqlType, Self::Backend>,
    {
        self.bind_as::<T::SqlType, T>(value)
    }

    fn bind_as<ST, T>(self, value: T) -> Self
    where
        Self::Backend: HasSqlType<ST>,
        T: ToSql<ST, Self::Backend>;

    fn finish(self, conn: &mut <Self::Backend as Backend>::RawConnection);

    #[inline]
    fn execute<'c, C>(self, executor: &'c C) -> BoxFuture<'c, io::Result<u64>>
    where
        Self: 'c + 'q,
        C: Executor<Backend = Self::Backend>,
    {
        executor.execute(self)
    }

    #[inline]
    fn fetch<'c, A: 'c, T: 'c, C>(self, executor: &'c C) -> BoxStream<'c, io::Result<T>>
    where
        Self: 'c + 'q,
        C: Executor<Backend = Self::Backend>,
        T: FromRow<A, Self::Backend> + Send + Unpin,
    {
        executor.fetch(self)
    }

    #[inline]
    fn fetch_optional<'c, A: 'c, T: 'c, C>(
        self,
        executor: &'c C,
    ) -> BoxFuture<'c, io::Result<Option<T>>>
    where
        Self: 'c + 'q,
        C: Executor<Backend = Self::Backend>,
        T: FromRow<A, Self::Backend>,
    {
        executor.fetch_optional(self)
    }
}

#[inline]
pub fn query<'q, Q>(query: &'q str) -> Q
where
    Q: Query<'q>,
{
    Q::new(query)
}
