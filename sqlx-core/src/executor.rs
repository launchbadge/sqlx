#[cfg(feature = "async")]
use futures_core::future::BoxFuture;

use crate::{Database, Result, Runtime};

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
    type Database: Database<Rt>;

    /// Execute the SQL query and return information about the result, including
    /// the number of rows affected, if any.
    #[cfg(feature = "async")]
    fn execute<'x, 'e, 'q>(
        &'e mut self,
        sql: &'q str,
    ) -> BoxFuture<'x, Result<<Self::Database as Database<Rt>>::QueryResult>>
    where
        Rt: crate::Async,
        'e: 'x,
        'q: 'x;

    #[cfg(feature = "async")]
    fn fetch_all<'x, 'e, 'q>(
        &'e mut self,
        sql: &'q str,
    ) -> BoxFuture<'x, Result<Vec<<Self::Database as Database<Rt>>::Row>>>
    where
        Rt: crate::Async,
        'e: 'x,
        'q: 'x;

    #[cfg(feature = "async")]
    fn fetch_optional<'x, 'e, 'q>(
        &'e mut self,
        sql: &'q str,
    ) -> BoxFuture<'x, Result<Option<<Self::Database as Database<Rt>>::Row>>>
    where
        Rt: crate::Async,
        'e: 'x,
        'q: 'x;

    #[cfg(feature = "async")]
    fn fetch_one<'x, 'e, 'q>(
        &'e mut self,
        sql: &'q str,
    ) -> BoxFuture<'x, Result<<Self::Database as Database<Rt>>::Row>>
    where
        Rt: crate::Async,
        'e: 'x,
        'q: 'x;
}
