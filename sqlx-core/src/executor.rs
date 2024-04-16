use crate::database::Database;
use crate::describe::Describe;
use crate::error::Error;

use futures_core::future::BoxFuture;
use std::fmt::Debug;
use std::marker::PhantomData;
use crate::query_string::QueryString;
use crate::result_set::ResultSet;

/// A type that contains or can provide a database
/// connection to use for executing queries against the database.
///
/// No guarantees are provided that successive queries run on the same
/// physical database connection.
///
/// A [`Connection`](crate::connection::Connection) is an `Executor` that guarantees that
/// successive queries are ran on the same physical database connection.
///
/// Implemented for the following:
///
///  * [`&Pool`](super::pool::Pool)
///  * [`&mut Connection`](super::connection::Connection)
///
/// The [`Executor`](crate::Executor) impls for [`Transaction`](crate::Transaction)
/// and [`PoolConnection`](crate::pool::PoolConnection) have been deleted because they
/// cannot exist in the new crate architecture without rewriting the Executor trait entirely.
/// To fix this breakage, simply add a dereference where an impl [`Executor`](crate::Executor) is expected, as
/// they both dereference to the inner connection type which will still implement it:
/// * `&mut transaction` -> `&mut *transaction`
/// * `&mut connection` -> `&mut *connection`
///
pub trait Executor<'c>: Send + Debug + Sized {
    type Database: Database;
    type ResultSet: ResultSet<
        Database = Self::Database,
        Row = <Self::Database as Database>::Row
    >;

    /// Execute a query as an implicitly prepared statement.
    async fn execute_prepared(&mut self, params: ExecutePrepared<'_, Self::Database>) -> Self::ResultSet;

    /// Execute raw SQL without creating a prepared statement.
    ///
    /// The SQL string may contain multiple statements separated by semicolons (`;`)
    /// as well as DDL (`CREATE TABLE`, `ALTER TABLE`, etc.).
    async fn execute_raw(&mut self, params: ExecuteRaw<'_, Self::Database>) -> Self::ResultSet;

    /// Prepare the SQL query to inspect the type information of its parameters
    /// and results.
    ///
    /// Be advised that when using the `query`, `query_as`, or `query_scalar` functions, the query
    /// is transparently prepared and executed.
    ///
    /// This explicit API is provided to allow access to the statement metadata available after
    /// it prepared but before the first row is returned.
    #[inline]
    async fn prepare<'e, 'q: 'e>(
        self,
        query: &'q str,
    ) -> Result<<Self::Database as Database>::Statement<'q>, Error>
    where
        'c: 'e,
    {
        self.prepare_with(query, &[]).await
    }

    /// Prepare the SQL query, with parameter type information, to inspect the
    /// type information about its parameters and results.
    ///
    /// Only some database drivers (PostgreSQL, MSSQL) can take advantage of
    /// this extra information to influence parameter type inference.
    async fn prepare_with<'e, 'q: 'e>(
        self,
        sql: &'q str,
        parameters: &'e [<Self::Database as Database>::TypeInfo],
    ) -> Result<<Self::Database as Database>::Statement<'q>, Error>
    where
        'c: 'e;

    /// Describe the SQL query and return type information about its parameters
    /// and results.
    ///
    /// This is used by compile-time verification in the query macros to
    /// power their type inference.
    #[doc(hidden)]
    fn describe<'e, 'q: 'e>(
        self,
        sql: &'q str,
    ) -> BoxFuture<'e, Result<Describe<Self::Database>, Error>>
    where
        'c: 'e;
}

/// Arguments struct for [`Executor::execute_prepared()`].
pub struct ExecutePrepared<'q, DB: Database> {
    /// The SQL string to execute.
    pub query: QueryString<'_>,
    /// The bind arguments for the query string; must match the number of placeholders.
    pub arguments: <DB as Database>::Arguments<'q>,
    /// The maximum number of rows to return.
    ///
    /// Set to `Some(0)` to just get the result.
    pub limit: Option<u64>,
    /// The number of rows to request from the database at a time.
    ///
    /// This is the maximum number of rows that will be buffered in-memory.
    ///
    /// This will also be the maximum number of rows that need to be read and discarded should the
    /// [`ResultSet`] be dropped early.
    pub buffer: Option<usize>,
    /// If `true`, prepare the statement with a name and cache it for later re-use.
    pub persistent: bool,
    _db: PhantomData<DB>
}

/// Arguments struct for [`Executor::execute_raw()`].
pub struct ExecuteRaw<'q, DB: Database> {
    /// The SQL string to execute.
    pub query: QueryString<'_>,
    /// The maximum number of rows to return.
    ///
    /// Set to `Some(0)` to just get the result.
    pub limit: Option<u64>,
    /// The number of rows to request from the database at a time.
    ///
    /// This is the maximum number of rows that will be buffered in-memory.
    ///
    /// This will also be the maximum number of rows that need to be read and discarded should the
    /// [`ResultSet`] be dropped early.
    pub buffer: Option<usize>,
    _db: PhantomData<DB>
}
