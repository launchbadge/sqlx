use crate::{
    backend::Backend, error::Error, executor::Executor, query::QueryParameters, row::FromSqlRow,
    serialize::ToSql, types::HasSqlType,
};
use futures_core::{future::BoxFuture, stream::BoxStream};

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
    /// Bind a value for use with this SQL query.
    ///
    /// # Safety
    ///
    /// This function should be used with care, as SQLx cannot validate
    /// that the value is of the right type nor can it validate that you have
    /// passed the correct number of parameters.
    pub fn bind<T>(mut self, value: T) -> Self
    where
        DB: HasSqlType<T>,
        T: ToSql<DB>,
    {
        self.params.bind(value);
        self
    }

    pub fn execute<E>(self, executor: &'q mut E) -> BoxFuture<'q, Result<u64, Error>>
    where
        E: Executor<Backend = DB>,
        DB::QueryParameters: 'q,
    {
        executor.execute(self.query, self.params)
    }

    pub fn fetch<E, T: 'q>(self, executor: &'q mut E) -> BoxStream<'q, Result<T, Error>>
    where
        E: Executor<Backend = DB>,
        T: FromSqlRow<DB> + Send + Unpin,
        DB::QueryParameters: 'q,
    {
        executor.fetch(self.query, self.params)
    }

    pub fn fetch_all<E, T: 'q>(self, executor: &'q mut E) -> BoxFuture<'q, Result<Vec<T>, Error>>
    where
        E: Executor<Backend = DB>,
        T: FromSqlRow<DB> + Send + Unpin,
        DB::QueryParameters: 'q,
    {
        executor.fetch_all(self.query, self.params)
    }

    pub fn fetch_optional<E, T: 'q>(
        self,
        executor: &'q mut E,
    ) -> BoxFuture<'q, Result<Option<T>, Error>>
    where
        E: Executor<Backend = DB>,
        T: FromSqlRow<DB> + Send,
        DB::QueryParameters: 'q,
    {
        executor.fetch_optional(self.query, self.params)
    }

    pub fn fetch_one<E, T: 'q>(self, executor: &'q mut E) -> BoxFuture<'q, Result<T, Error>>
    where
        E: Executor<Backend = DB>,
        T: FromSqlRow<DB> + Send + Unpin,
        DB::QueryParameters: 'q,
    {
        executor.fetch_one(self.query, self.params)
    }
}

/// Construct a full SQL query using raw SQL.
#[inline]
pub fn query<DB>(query: &str) -> SqlQuery<'_, DB>
where
    DB: Backend,
{
    SqlQuery {
        query,
        params: DB::QueryParameters::new(),
    }
}
