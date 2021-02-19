#[cfg(feature = "async")]
use futures_util::future::{self, BoxFuture, FutureExt, TryFutureExt};

use crate::{Arguments, Database, Execute, Runtime};

/// Describes a type that can execute SQL queries on a self-provided database connection.
///
/// No guarantees are provided that successive queries run on the same physical
/// database connection.
///
/// A [`Connection`] is an `Executor` that guarantees that successive queries are ran on the
/// same physical database connection.
///
#[allow(clippy::type_complexity)]
pub trait Executor<Rt: Runtime> {
    type Database: Database;

    /// Execute the SQL query and return information about the result, including
    /// the number of rows affected, if any.
    #[cfg(feature = "async")]
    fn execute<'x, 'e, 'q, 'a, E>(
        &'e mut self,
        query: E,
    ) -> BoxFuture<'x, crate::Result<<Self::Database as Database>::QueryResult>>
    where
        Rt: crate::Async,
        E: 'x + Execute<'q, 'a, Self::Database>,
        'e: 'x,
        'q: 'x,
        'a: 'x;

    #[cfg(feature = "async")]
    fn fetch_all<'x, 'e, 'q, 'a, E>(
        &'e mut self,
        query: E,
    ) -> BoxFuture<'x, crate::Result<Vec<<Self::Database as Database>::Row>>>
    where
        Rt: crate::Async,
        E: 'x + Execute<'q, 'a, Self::Database>,
        'e: 'x,
        'q: 'x,
        'a: 'x;

    #[cfg(feature = "async")]
    fn fetch_optional<'x, 'e, 'q, 'a, E>(
        &'e mut self,
        query: E,
    ) -> BoxFuture<'x, crate::Result<Option<<Self::Database as Database>::Row>>>
    where
        Rt: crate::Async,
        E: 'x + Execute<'q, 'a, Self::Database>,
        'e: 'x,
        'q: 'x,
        'a: 'x;

    #[cfg(feature = "async")]
    fn fetch_one<'x, 'e, 'q, 'a, E>(
        &'e mut self,
        query: E,
    ) -> BoxFuture<'x, crate::Result<<Self::Database as Database>::Row>>
    where
        Rt: crate::Async,
        E: 'x + Execute<'q, 'a, Self::Database>,
        'e: 'x,
        'q: 'x,
        'a: 'x,
    {
        self.fetch_optional(query)
            .and_then(|maybe_row| match maybe_row {
                Some(row) => future::ok(row),
                None => future::err(crate::Error::RowNotFound),
            })
            .boxed()
    }
}

impl<Rt: Runtime, X: Executor<Rt>> Executor<Rt> for &'_ mut X {
    type Database = X::Database;

    #[cfg(feature = "async")]
    fn execute<'x, 'e, 'q, 'a, E>(
        &'e mut self,
        query: E,
    ) -> BoxFuture<'x, crate::Result<<Self::Database as Database>::QueryResult>>
    where
        Rt: crate::Async,
        E: 'x + Execute<'q, 'a, Self::Database>,
        'e: 'x,
        'q: 'x,
        'a: 'x,
    {
        (**self).execute(query)
    }

    #[cfg(feature = "async")]
    fn fetch_all<'x, 'e, 'q, 'a, E>(
        &'e mut self,
        query: E,
    ) -> BoxFuture<'x, crate::Result<Vec<<Self::Database as Database>::Row>>>
    where
        Rt: crate::Async,
        E: 'x + Execute<'q, 'a, Self::Database>,
        'e: 'x,
        'q: 'x,
        'a: 'x,
    {
        (**self).fetch_all(query)
    }

    #[cfg(feature = "async")]
    fn fetch_optional<'x, 'e, 'q, 'a, E>(
        &'e mut self,
        query: E,
    ) -> BoxFuture<'x, crate::Result<Option<<Self::Database as Database>::Row>>>
    where
        Rt: crate::Async,
        E: 'x + Execute<'q, 'a, Self::Database>,
        'e: 'x,
        'q: 'x,
        'a: 'x,
    {
        (**self).fetch_optional(query)
    }
}
