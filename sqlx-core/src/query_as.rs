use std::marker::PhantomData;

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
use crate::query::{query, Map, Query};
use crate::row::FromRow;

/// Raw SQL query with bind parameters, mapped to a concrete type using [`FromRow`].
/// Returned from [`query_as`].
#[must_use = "query must be executed to affect database"]
pub struct QueryAs<'q, DB: Database, O> {
    pub(crate) inner: Query<'q, DB>,
    output: PhantomData<O>,
}

impl<'q, DB, O: Send> Execute<'q, DB> for QueryAs<'q, DB, O>
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
impl<'q, DB, O> QueryAs<'q, DB, O>
where
    DB: Database,
    O: Send + Unpin + for<'r> FromRow<'r, DB::Row>,
{
    /// Bind a value for use with this SQL query.
    ///
    /// See [`Query::bind`](crate::query::Query::bind).
    #[inline]
    pub fn bind<T: Encode<DB>>(mut self, value: T) -> Self {
        self.inner.arguments.add(value);
        self
    }

    /// Execute the query and return the generated results as a stream.
    pub fn fetch<'c, E>(self, executor: E) -> BoxStream<'c, Result<O, Error>>
    where
        'q: 'c,
        E: 'c + Executor<'c, Database = DB>,
        DB: 'c,
        O: 'c,
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
    pub fn fetch_many<'c, E>(self, executor: E) -> BoxStream<'c, Result<Either<u64, O>, Error>>
    where
        'q: 'c,
        E: 'c + Executor<'c, Database = DB>,
        DB: 'c,
        O: 'c,
    {
        Box::pin(try_stream! {
            let mut s = executor.fetch_many(self.inner);
            while let Some(v) = s.try_next().await? {
                match v {
                    Either::Left(v) => yield Either::Left(v),
                    Either::Right(row) => {
                        let mapped = O::from_row(&row)?;
                        yield Either::Right(mapped);
                    }
                }
            }
        })
    }

    /// Execute the query and return all the generated results, collected into a [`Vec`].
    #[inline]
    pub fn fetch_all<'c, E>(self, executor: E) -> BoxFuture<'c, Result<Vec<O>, Error>>
    where
        'q: 'c,
        E: 'c + Executor<'c, Database = DB>,
        DB: 'c,
        O: 'c,
    {
        self.fetch(executor).try_collect().boxed()
    }

    /// Execute the query and returns exactly one row.
    pub fn fetch_one<'c, E>(self, executor: E) -> BoxFuture<'c, Result<O, Error>>
    where
        'q: 'c,
        E: 'c + Executor<'c, Database = DB>,
        DB: 'c,
        O: 'c,
    {
        self.fetch_optional(executor)
            .and_then(|row| match row {
                Some(row) => future::ok(row),
                None => future::err(Error::RowNotFound),
            })
            .boxed()
    }

    /// Execute the query and returns at most one row.
    pub fn fetch_optional<'c, E>(self, executor: E) -> BoxFuture<'c, Result<Option<O>, Error>>
    where
        'q: 'c,
        E: 'c + Executor<'c, Database = DB>,
        DB: 'c,
        O: 'c,
    {
        Box::pin(async move {
            let row = executor.fetch_optional(self.inner).await?;
            if let Some(row) = row {
                O::from_row(&row).map(Some)
            } else {
                Ok(None)
            }
        })
    }
}

/// Construct a raw SQL query that is mapped to a concrete type
/// using [`FromRow`](crate::row::FromRow).
#[inline]
pub fn query_as<DB, O>(sql: &str) -> QueryAs<DB, O>
where
    DB: Database,
    O: for<'r> FromRow<'r, DB::Row>,
{
    QueryAs {
        inner: query(sql),
        output: PhantomData,
    }
}
