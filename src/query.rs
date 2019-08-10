use crate::{
    backend::Backend,
    row::FromRow,
    serialize::ToSql,
    types::{AsSqlType, HasSqlType},
};
use futures::{future::BoxFuture, stream::BoxStream};
use std::io;

pub trait Query<'c, 'q> {
    type Backend: Backend;

    #[inline]
    fn bind<T>(self, value: T) -> Self
    where
        Self: Sized,
        Self::Backend: HasSqlType<<T as AsSqlType<Self::Backend>>::SqlType>,
        T: AsSqlType<Self::Backend>
            + ToSql<<T as AsSqlType<Self::Backend>>::SqlType, Self::Backend>,
    {
        self.bind_as::<T::SqlType, T>(value)
    }

    fn bind_as<ST, T>(self, value: T) -> Self
    where
        Self: Sized,
        Self::Backend: HasSqlType<ST>,
        T: ToSql<ST, Self::Backend>;

    fn execute(self) -> BoxFuture<'c, io::Result<u64>>;

    fn fetch<A: 'c, T: 'c>(self) -> BoxStream<'c, io::Result<T>>
    where
        T: FromRow<A, Self::Backend>;

    fn fetch_optional<A: 'c, T: 'c>(self) -> BoxFuture<'c, io::Result<Option<T>>>
    where
        T: FromRow<A, Self::Backend>;
}
