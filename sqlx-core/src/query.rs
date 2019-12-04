use crate::{backend::Backend, encode::Encode, error::Error, executor::Executor, params::{IntoQueryParameters, QueryParameters}, row::FromRow, types::HasSqlType, Row, Decode};
use bitflags::_core::marker::PhantomData;
use futures_core::{future::BoxFuture, stream::BoxStream};

pub struct Query<'q, DB, P = <DB as Backend>::QueryParameters, R = <DB as Backend>::Row>
where
    DB: Backend,
{
    query: &'q str,
    params: P,
    record: PhantomData<R>,
    backend: PhantomData<DB>,
}

impl<'q, DB, P: 'q, R: 'q> Query<'q, DB, P, R>
where
    DB: Backend,
    DB::QueryParameters: 'q,
    P: IntoQueryParameters<DB> + Send,
    R: FromRow<DB> + Send + Unpin,
{
    #[inline]
    pub fn execute<E>(self, executor: &'q mut E) -> BoxFuture<'q, crate::Result<u64>>
    where
        E: Executor<Backend = DB>,
    {
        executor.execute(self.query, self.params.into_params())
    }

    pub fn fetch<E>(self, executor: &'q mut E) -> BoxStream<'q, crate::Result<R>>
    where
        E: Executor<Backend = DB>,
    {
        executor.fetch(self.query, self.params.into_params())
    }

    pub fn fetch_all<E>(self, executor: &'q mut E) -> BoxFuture<'q, crate::Result<Vec<R>>>
    where
        E: Executor<Backend = DB>,
    {
        executor.fetch_all(self.query, self.params.into_params())
    }

    pub fn fetch_optional<E>(self, executor: &'q mut E) -> BoxFuture<'q, crate::Result<Option<R>>>
    where
        E: Executor<Backend = DB>,
    {
        executor.fetch_optional(self.query, self.params.into_params())
    }

    pub fn fetch_one<E>(self, executor: &'q mut E) -> BoxFuture<'q, crate::Result<R>>
    where
        E: Executor<Backend = DB>,
    {
        executor.fetch_one(self.query, self.params.into_params())
    }
}

impl<'q, DB> Query<'q, DB>
where
    DB: Backend,
{
    /// Bind a value for use with this SQL query.
    ///
    /// # Logic Safety
    ///
    /// This function should be used with care, as SQLx cannot validate
    /// that the value is of the right type nor can it validate that you have
    /// passed the correct number of parameters.
    pub fn bind<T>(mut self, value: T) -> Self
    where
        DB: HasSqlType<T>,
        T: Encode<DB>,
    {
        self.params.bind(value);
        self
    }

    /// Bind all query parameters at once.
    ///
    /// If any parameters were previously bound with `.bind()` they are discarded.
    ///
    /// # Logic Safety
    ///
    /// This function should be used with care, as SQLx cannot validate
    /// that the value is of the right type nor can it validate that you have
    /// passed the correct number of parameters.
    pub fn bind_all<I>(self, values: I) -> Query<'q, DB, I> where I: IntoQueryParameters<DB> {
        Query {
            query: self.query,
            params: values,
            record: PhantomData,
            backend: PhantomData
        }
    }
}
//noinspection RsSelfConvention
impl<'q, DB, I, R> Query<'q, DB, I, R> where DB: Backend {

    /// Change the expected output type of the query to a single scalar value.
    pub fn as_scalar<R_>(self) -> Query<'q, DB, I, R_> where R_: Decode<DB> {
        Query {
            query: self.query,
            params: self.params,
            record: PhantomData,
            backend: PhantomData,
        }
    }

    /// Change the expected output of the query to a new type implementing `FromRow`.
    pub fn as_record<R_>(self) -> Query<'q, DB, I, R_> where R_: FromRow<DB> {
        Query {
            query: self.query,
            params: self.params,
            record: PhantomData,
            backend: PhantomData,
        }
    }
}

/// Construct a full SQL query using raw SQL.
#[inline]
pub fn query<DB>(query: &str) -> Query<'_, DB>
where
    DB: Backend,
{
    Query {
        query,
        params: Default::default(),
        record: PhantomData,
        backend: PhantomData,
    }
}
