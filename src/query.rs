use crate::{
    backend::Backend, error::Error, executor::Executor, row::FromSqlRow, serialize::ToSql,
    types::HasSqlType,
};
use futures_core::{future::BoxFuture, stream::BoxStream};
use std::io;

pub trait QueryParameters: Send {
    type Backend: Backend;

    fn new() -> Self
    where
        Self: Sized;

    fn bind<T>(&mut self, value: T)
    where
        Self::Backend: HasSqlType<T>,
        T: ToSql<Self::Backend>;
}

pub struct SqlQuery<'q, DB>
where
    DB: Backend,
{
    query: &'q str,
    params: DB::QueryParameters,
}

impl<'q, DB> SqlQuery<'q, DB>
where
    DB: Backend,
{
    #[inline]
    pub fn new(query: &'q str) -> Self {
        Self {
            query,
            params: DB::QueryParameters::new(),
        }
    }

    #[inline]
    pub fn bind<T>(mut self, value: T) -> Self
    where
        DB: HasSqlType<T>,
        T: ToSql<DB>,
    {
        self.params.bind(value);
        self
    }

    // TODO: These methods should go on a [Execute] trait (so more execut-able things can be defined)

    #[inline]
    pub fn execute<E>(self, executor: &'q E) -> BoxFuture<'q, Result<u64, Error>>
    where
        E: Executor<Backend = DB>,
    {
        executor.execute(self.query, self.params)
    }

    #[inline]
    pub fn fetch<E, T: 'q>(self, executor: &'q E) -> BoxStream<'q, Result<T, Error>>
    where
        E: Executor<Backend = DB>,
        T: FromSqlRow<DB> + Send + Unpin,
    {
        executor.fetch(self.query, self.params)
    }

    #[inline]
    pub fn fetch_optional<E, T: 'q>(
        self,
        executor: &'q E,
    ) -> BoxFuture<'q, Result<Option<T>, Error>>
    where
        E: Executor<Backend = DB>,
        T: FromSqlRow<DB>,
    {
        executor.fetch_optional(self.query, self.params)
    }
}

/// Construct a full SQL query using raw SQL.
#[inline]
pub fn query<'q, DB>(query: &'q str) -> SqlQuery<'q, DB>
where
    DB: Backend,
{
    SqlQuery::new(query)
}
