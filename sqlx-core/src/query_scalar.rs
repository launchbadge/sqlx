use async_stream::try_stream;
use either::Either;
use futures_core::future::BoxFuture;
use futures_core::stream::BoxStream;
use futures_util::{future, FutureExt, StreamExt, TryFutureExt, TryStreamExt};

use crate::arguments::Arguments;
use crate::database::{Database, HasArguments};
use crate::encode::Encode;
use crate::error::Error;
use crate::executor::{Execute, Executor};
use crate::from_row::FromRow;
use crate::query::{Map, Query};
use crate::query_as::{query_as, QueryAs};

/// Raw SQL query with bind parameters, mapped to a concrete type using [`FromRow`] on `(O,)`.
/// Returned from [`query_scalar`].
#[must_use = "query must be executed to affect database"]
pub struct QueryScalar<'q, DB: Database, O> {
    inner: QueryAs<'q, DB, (O,)>,
}

impl<'q, DB, O: Send> Execute<'q, DB> for QueryScalar<'q, DB, O>
where
    DB: Database,
{
    #[inline]
    fn query(&self) -> &'q str {
        self.inner.query()
    }

    #[inline]
    fn take_arguments(&mut self) -> Option<<DB as HasArguments<'q>>::Arguments> {
        self.inner.take_arguments()
    }
}

// FIXME: This is very close, nearly 1:1 with `Map`
// noinspection DuplicatedCode
impl<'q, DB, O> QueryScalar<'q, DB, O>
where
    DB: Database,
    O: Send + Unpin,
    (O,): Send + Unpin + for<'r> FromRow<'r, DB::Row>,
{
    /// Bind a value for use with this SQL query.
    ///
    /// See [`Query::bind`](crate::query::Query::bind).
    #[inline]
    pub fn bind<T: 'q + Encode<'q, DB>>(mut self, value: T) -> Self {
        self.inner.inner.arguments.add(value);
        self
    }

    /// Execute the query and return the generated results as a stream.
    #[inline]
    pub fn fetch<'c, E>(self, executor: E) -> BoxStream<'c, Result<O, Error>>
    where
        'q: 'c,
        E: 'c + Executor<'c, Database = DB>,
        DB: 'c,
        O: 'c,
    {
        self.inner.fetch(executor).map_ok(|it| it.0).boxed()
    }

    /// Execute multiple queries and return the generated results as a stream
    /// from each query, in a stream.
    #[inline]
    pub fn fetch_many<'c, E>(self, executor: E) -> BoxStream<'c, Result<Either<u64, O>, Error>>
    where
        'q: 'c,
        E: 'c + Executor<'c, Database = DB>,
        DB: 'c,
        O: 'c,
    {
        self.inner
            .fetch_many(executor)
            .map_ok(|v| v.map_right(|it| it.0))
            .boxed()
    }

    /// Execute the query and return all the generated results, collected into a [`Vec`].
    #[inline]
    pub async fn fetch_all<'c, E>(self, executor: E) -> Result<Vec<O>, Error>
    where
        'q: 'c,
        E: 'c + Executor<'c, Database = DB>,
        DB: 'c,
        (O,): 'c,
    {
        self.inner
            .fetch(executor)
            .map_ok(|it| it.0)
            .try_collect()
            .await
    }

    /// Execute the query and returns exactly one row.
    #[inline]
    pub async fn fetch_one<'c, E>(self, executor: E) -> Result<O, Error>
    where
        'q: 'c,
        E: 'c + Executor<'c, Database = DB>,
        DB: 'c,
        O: 'c,
    {
        self.inner.fetch_one(executor).map_ok(|it| it.0).await
    }

    /// Execute the query and returns at most one row.
    #[inline]
    pub async fn fetch_optional<'c, E>(self, executor: E) -> Result<Option<O>, Error>
    where
        'q: 'c,
        E: 'c + Executor<'c, Database = DB>,
        DB: 'c,
        O: 'c,
    {
        Ok(self.inner.fetch_optional(executor).await?.map(|it| it.0))
    }
}

/// Construct a raw SQL query that is mapped to a concrete type
/// using [`FromRow`](crate::row::FromRow) on `(O,)`.
#[inline]
pub fn query_scalar<DB, O>(sql: &str) -> QueryScalar<DB, O>
where
    DB: Database,
    (O,): for<'r> FromRow<'r, DB::Row>,
{
    QueryScalar {
        inner: query_as(sql),
    }
}
