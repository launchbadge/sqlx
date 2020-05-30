use std::marker::PhantomData;

use async_stream::try_stream;
use either::Either;
use futures_core::stream::BoxStream;
use futures_util::{future, StreamExt, TryFutureExt, TryStreamExt};

use crate::arguments::{Arguments, IntoArguments};
use crate::database::{Database, HasArguments};
use crate::encode::Encode;
use crate::error::Error;
use crate::executor::{Execute, Executor};

/// Raw SQL query with bind parameters. Returned by [`query`][crate::query::query].
#[must_use = "query must be executed to affect database"]
pub struct Query<'q, DB: Database, A> {
    pub(crate) query: &'q str,
    pub(crate) arguments: Option<A>,
    pub(crate) database: PhantomData<DB>,
}

/// SQL query that will map its results to owned Rust types.
///
/// Returned by [Query::try_map], `query!()`, etc. Has most of the same methods as [Query] but
/// the return types are changed to reflect the mapping. However, there is no equivalent of
/// [Query::execute] as it doesn't make sense to map the result type and then ignore it.
///
/// [Query::bind] is also omitted; stylistically we recommend placing your `.bind()` calls
/// before `.try_map()`.
#[must_use = "query must be executed to affect database"]
pub struct Map<'q, DB: Database, F, A> {
    inner: Query<'q, DB, A>,
    mapper: F,
}

impl<'q, DB, A> Execute<'q, DB> for Query<'q, DB, A>
where
    DB: Database,
    A: Send + IntoArguments<'q, DB>,
{
    #[inline]
    fn query(&self) -> &'q str {
        self.query
    }

    #[inline]
    fn take_arguments(&mut self) -> Option<<DB as HasArguments<'q>>::Arguments> {
        self.arguments.take().map(IntoArguments::into_arguments)
    }
}

impl<'q, DB: Database> Query<'q, DB, <DB as HasArguments<'q>>::Arguments> {
    /// Bind a value for use with this SQL query.
    ///
    /// If the number of times this is called does not match the number of bind parameters that
    /// appear in the query (`?` for most SQL flavors, `$1 .. $N` for Postgres) then an error
    /// will be returned when this query is executed.
    ///
    /// There is no validation that the value is of the type expected by the query. Most SQL
    /// flavors will perform type coercion (Postgres will return a database error).
    pub fn bind<T: 'q + Encode<'q, DB>>(mut self, value: T) -> Self {
        if let Some(arguments) = &mut self.arguments {
            arguments.add(value);
        }

        self
    }
}

impl<'q, DB, A: Send> Query<'q, DB, A>
where
    DB: Database,
    A: 'q + IntoArguments<'q, DB>,
{
    /// Map each row in the result to another type.
    ///
    /// See [`try_map`](Query::try_map) for a fallible version of this method.
    ///
    /// The [`query_as`](crate::query_as::query_as) method will construct a mapped query using
    /// a [`FromRow`](crate::row::FromRow) implementation.
    #[inline]
    pub fn map<F, O>(self, f: F) -> Map<'q, DB, impl Fn(DB::Row) -> Result<O, Error>, A>
    where
        F: Fn(DB::Row) -> O,
    {
        self.try_map(move |row| Ok(f(row)))
    }

    /// Map each row in the result to another type.
    ///
    /// The [`query_as`](crate::query_as::query_as) method will construct a mapped query using
    /// a [`FromRow`](crate::row::FromRow) implementation.
    #[inline]
    pub fn try_map<F, O>(self, f: F) -> Map<'q, DB, F, A>
    where
        F: Fn(DB::Row) -> Result<O, Error>,
    {
        Map {
            inner: self,
            mapper: f,
        }
    }

    /// Execute the query and return the total number of rows affected.
    #[inline]
    pub async fn execute<'e, 'c: 'e, E>(self, executor: E) -> Result<u64, Error>
    where
        'q: 'e,
        A: 'e,
        E: Executor<'c, Database = DB>,
    {
        executor.execute(self).await
    }

    /// Execute multiple queries and return the rows affected from each query, in a stream.
    #[inline]
    pub async fn execute_many<'e, 'c: 'e, E>(self, executor: E) -> BoxStream<'e, Result<u64, Error>>
    where
        'q: 'e,
        A: 'e,
        E: Executor<'c, Database = DB>,
    {
        executor.execute_many(self)
    }

    /// Execute the query and return the generated results as a stream.
    #[inline]
    pub fn fetch<'e, 'c: 'e, E>(self, executor: E) -> BoxStream<'e, Result<DB::Row, Error>>
    where
        'q: 'e,
        A: 'e,
        E: Executor<'c, Database = DB>,
    {
        executor.fetch(self)
    }

    /// Execute multiple queries and return the generated results as a stream
    /// from each query, in a stream.
    #[inline]
    pub fn fetch_many<'e, 'c: 'e, E>(
        self,
        executor: E,
    ) -> BoxStream<'e, Result<Either<u64, DB::Row>, Error>>
    where
        'q: 'e,
        A: 'e,
        E: Executor<'c, Database = DB>,
    {
        executor.fetch_many(self)
    }

    /// Execute the query and return all the generated results, collected into a [`Vec`].
    #[inline]
    pub async fn fetch_all<'e, 'c: 'e, E>(self, executor: E) -> Result<Vec<DB::Row>, Error>
    where
        'q: 'e,
        A: 'e,
        E: Executor<'c, Database = DB>,
    {
        executor.fetch_all(self).await
    }

    /// Execute the query and returns exactly one row.
    #[inline]
    pub async fn fetch_one<'e, 'c: 'e, E>(self, executor: E) -> Result<DB::Row, Error>
    where
        'q: 'e,
        A: 'e,
        E: Executor<'c, Database = DB>,
    {
        executor.fetch_one(self).await
    }

    /// Execute the query and returns at most one row.
    #[inline]
    pub async fn fetch_optional<'e, 'c: 'e, E>(self, executor: E) -> Result<Option<DB::Row>, Error>
    where
        'q: 'e,
        A: 'e,
        E: Executor<'c, Database = DB>,
    {
        executor.fetch_optional(self).await
    }
}

impl<'q, DB, F: Send, A: Send> Execute<'q, DB> for Map<'q, DB, F, A>
where
    DB: Database,
    A: IntoArguments<'q, DB>,
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

impl<'q, DB, F, O, A> Map<'q, DB, F, A>
where
    DB: Database,
    F: Send + Sync + Fn(DB::Row) -> Result<O, Error>,
    O: Send + Unpin,
    A: 'q + Send + IntoArguments<'q, DB>,
{
    /// Execute the query and return the generated results as a stream.
    pub fn fetch<'e, 'c: 'e, E>(self, executor: E) -> BoxStream<'e, Result<O, Error>>
    where
        'q: 'e,
        E: 'e + Executor<'c, Database = DB>,
        DB: 'e,
        F: 'e,
        O: 'e,
    {
        self.fetch_many(executor)
            .try_filter_map(|step| async move {
                Ok(match step {
                    Either::Left(_) => None,
                    Either::Right(o) => Some(o),
                })
            })
            .boxed()
    }

    /// Execute multiple queries and return the generated results as a stream
    /// from each query, in a stream.
    pub fn fetch_many<'e, 'c: 'e, E>(
        self,
        executor: E,
    ) -> BoxStream<'e, Result<Either<u64, O>, Error>>
    where
        'q: 'e,
        E: 'e + Executor<'c, Database = DB>,
        DB: 'e,
        F: 'e,
        O: 'e,
    {
        Box::pin(try_stream! {
            let mut s = executor.fetch_many(self.inner);
            while let Some(v) = s.try_next().await? {
                match v {
                    Either::Left(v) => yield Either::Left(v),
                    Either::Right(row) => {
                        let mapped = (self.mapper)(row)?;
                        yield Either::Right(mapped);
                    }
                }
            }
        })
    }

    /// Execute the query and return all the generated results, collected into a [`Vec`].
    pub async fn fetch_all<'e, 'c: 'e, E>(self, executor: E) -> Result<Vec<O>, Error>
    where
        'q: 'e,
        E: 'e + Executor<'c, Database = DB>,
        DB: 'e,
        F: 'e,
        O: 'e,
    {
        self.fetch(executor).try_collect().await
    }

    /// Execute the query and returns exactly one row.
    pub async fn fetch_one<'e, 'c: 'e, E>(self, executor: E) -> Result<O, Error>
    where
        'q: 'e,
        E: 'e + Executor<'c, Database = DB>,
        DB: 'e,
        F: 'e,
        O: 'e,
    {
        self.fetch_optional(executor)
            .and_then(|row| match row {
                Some(row) => future::ok(row),
                None => future::err(Error::RowNotFound),
            })
            .await
    }

    /// Execute the query and returns at most one row.
    pub async fn fetch_optional<'e, 'c: 'e, E>(self, executor: E) -> Result<Option<O>, Error>
    where
        'q: 'e,
        E: 'e + Executor<'c, Database = DB>,
        DB: 'e,
        F: 'e,
        O: 'e,
    {
        let row = executor.fetch_optional(self.inner).await?;

        if let Some(row) = row {
            (self.mapper)(row).map(Some)
        } else {
            Ok(None)
        }
    }
}

/// Make a SQL query.
#[inline]
pub fn query<'q, DB>(sql: &'q str) -> Query<DB, <DB as HasArguments<'q>>::Arguments>
where
    DB: Database,
{
    Query {
        database: PhantomData,
        arguments: Some(Default::default()),
        query: sql,
    }
}

/// Make a SQL query, with the given arguments.
#[inline]
pub fn query_with<'q, DB, A>(sql: &'q str, arguments: A) -> Query<'q, DB, A>
where
    DB: Database,
    A: IntoArguments<'q, DB>,
{
    Query {
        database: PhantomData,
        arguments: Some(arguments),
        query: sql,
    }
}
