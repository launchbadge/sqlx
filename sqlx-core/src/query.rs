use std::marker::PhantomData;

use either::Either;
use futures_core::stream::BoxStream;
use futures_util::{future, StreamExt, TryFutureExt, TryStreamExt};

use crate::arguments::{Arguments, IntoArguments};
use crate::database::{Database, HasArguments, HasStatement, HasStatementCache};
use crate::encode::Encode;
use crate::error::Error;
use crate::executor::{Execute, Executor};
use crate::statement::Statement;
use crate::types::Type;

/// Raw SQL query with bind parameters. Returned by [`query`][crate::query::query].
#[must_use = "query must be executed to affect database"]
pub struct Query<'q, DB: Database, A> {
    pub(crate) statement: Either<&'q str, &'q <DB as HasStatement<'q>>::Statement>,
    pub(crate) arguments: Option<A>,
    pub(crate) database: PhantomData<DB>,
    pub(crate) persistent: bool,
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
    fn sql(&self) -> &'q str {
        match self.statement {
            Either::Right(ref statement) => statement.sql(),
            Either::Left(sql) => sql,
        }
    }

    fn statement(&self) -> Option<&<DB as HasStatement<'q>>::Statement> {
        match self.statement {
            Either::Right(ref statement) => Some(&statement),
            Either::Left(_) => None,
        }
    }

    #[inline]
    fn take_arguments(&mut self) -> Option<<DB as HasArguments<'q>>::Arguments> {
        self.arguments.take().map(IntoArguments::into_arguments)
    }

    #[inline]
    fn persistent(&self) -> bool {
        self.persistent
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
    pub fn bind<T: 'q + Send + Encode<'q, DB> + Type<DB>>(mut self, value: T) -> Self {
        if let Some(arguments) = &mut self.arguments {
            arguments.add(value);
        }

        self
    }
}

impl<'q, DB, A> Query<'q, DB, A>
where
    DB: Database + HasStatementCache,
{
    /// If `true`, the statement will get prepared once and cached to the
    /// connection's statement cache.
    ///
    /// If queried once with the flag set to `true`, all subsequent queries
    /// matching the one with the flag will use the cached statement until the
    /// cache is cleared.
    ///
    /// Default: `true`.
    pub fn persistent(mut self, value: bool) -> Self {
        self.persistent = value;
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
    /// The [`query_as`](super::query_as::query_as) method will construct a mapped query using
    /// a [`FromRow`](super::from_row::FromRow) implementation.
    #[inline]
    pub fn map<F, O>(self, f: F) -> Map<'q, DB, impl TryMapRow<DB, Output = O>, A>
    where
        F: MapRow<DB, Output = O>,
        O: Unpin,
    {
        self.try_map(MapRowAdapter(f))
    }

    /// Map each row in the result to another type.
    ///
    /// The [`query_as`](super::query_as::query_as) method will construct a mapped query using
    /// a [`FromRow`](super::from_row::FromRow) implementation.
    #[inline]
    pub fn try_map<F>(self, f: F) -> Map<'q, DB, F, A>
    where
        F: TryMapRow<DB>,
    {
        Map {
            inner: self,
            mapper: f,
        }
    }

    /// Execute the query and return the total number of rows affected.
    #[inline]
    pub async fn execute<'e, 'c: 'e, E>(self, executor: E) -> Result<DB::Done, Error>
    where
        'q: 'e,
        A: 'e,
        E: Executor<'c, Database = DB>,
    {
        executor.execute(self).await
    }

    /// Execute multiple queries and return the rows affected from each query, in a stream.
    #[inline]
    pub async fn execute_many<'e, 'c: 'e, E>(
        self,
        executor: E,
    ) -> BoxStream<'e, Result<DB::Done, Error>>
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
    ) -> BoxStream<'e, Result<Either<DB::Done, DB::Row>, Error>>
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
    fn sql(&self) -> &'q str {
        self.inner.sql()
    }

    #[inline]
    fn statement(&self) -> Option<&<DB as HasStatement<'q>>::Statement> {
        self.inner.statement()
    }

    #[inline]
    fn take_arguments(&mut self) -> Option<<DB as HasArguments<'q>>::Arguments> {
        self.inner.take_arguments()
    }

    #[inline]
    fn persistent(&self) -> bool {
        self.inner.arguments.is_some()
    }
}

impl<'q, DB, F, O, A> Map<'q, DB, F, A>
where
    DB: Database,
    F: TryMapRow<DB, Output = O>,
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
        mut self,
        executor: E,
    ) -> BoxStream<'e, Result<Either<DB::Done, O>, Error>>
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
                r#yield!(match v {
                    Either::Left(v) => Either::Left(v),
                    Either::Right(row) => {
                        Either::Right(self.mapper.try_map_row(row)?)
                    }
                });
            }

            Ok(())
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
    pub async fn fetch_optional<'e, 'c: 'e, E>(mut self, executor: E) -> Result<Option<O>, Error>
    where
        'q: 'e,
        E: 'e + Executor<'c, Database = DB>,
        DB: 'e,
        F: 'e,
        O: 'e,
    {
        let row = executor.fetch_optional(self.inner).await?;

        if let Some(row) = row {
            self.mapper.try_map_row(row).map(Some)
        } else {
            Ok(None)
        }
    }
}

// A (hopefully) temporary workaround for an internal compiler error (ICE) involving higher-ranked
// trait bounds (HRTBs), associated types and closures.
//
// See https://github.com/rust-lang/rust/issues/62529

pub trait TryMapRow<DB: Database>: Send {
    type Output: Unpin;

    fn try_map_row(&mut self, row: DB::Row) -> Result<Self::Output, Error>;
}

pub trait MapRow<DB: Database>: Send {
    type Output: Unpin;

    fn map_row(&mut self, row: DB::Row) -> Self::Output;
}

// A private adapter that implements [MapRow] in terms of [TryMapRow]
// Just ends up Ok wrapping it

struct MapRowAdapter<F>(F);

impl<DB: Database, O, F> TryMapRow<DB> for MapRowAdapter<F>
where
    O: Unpin,
    F: MapRow<DB, Output = O>,
{
    type Output = O;

    fn try_map_row(&mut self, row: DB::Row) -> Result<Self::Output, Error> {
        Ok(self.0.map_row(row))
    }
}

// Make a SQL query from a statement.
pub(crate) fn query_statement<'q, DB>(
    statement: &'q <DB as HasStatement<'q>>::Statement,
) -> Query<'q, DB, <DB as HasArguments<'_>>::Arguments>
where
    DB: Database,
{
    Query {
        database: PhantomData,
        arguments: Some(Default::default()),
        statement: Either::Right(statement),
        persistent: true,
    }
}

// Make a SQL query from a statement, with the given arguments.
pub(crate) fn query_statement_with<'q, DB, A>(
    statement: &'q <DB as HasStatement<'q>>::Statement,
    arguments: A,
) -> Query<'q, DB, A>
where
    DB: Database,
    A: IntoArguments<'q, DB>,
{
    Query {
        database: PhantomData,
        arguments: Some(arguments),
        statement: Either::Right(statement),
        persistent: true,
    }
}

/// Make a SQL query.
pub fn query<DB>(sql: &str) -> Query<'_, DB, <DB as HasArguments<'_>>::Arguments>
where
    DB: Database,
{
    Query {
        database: PhantomData,
        arguments: Some(Default::default()),
        statement: Either::Left(sql),
        persistent: true,
    }
}

/// Make a SQL query, with the given arguments.
pub fn query_with<'q, DB, A>(sql: &'q str, arguments: A) -> Query<'q, DB, A>
where
    DB: Database,
    A: IntoArguments<'q, DB>,
{
    Query {
        database: PhantomData,
        arguments: Some(arguments),
        statement: Either::Left(sql),
        persistent: true,
    }
}

#[allow(unused_macros)]
macro_rules! impl_map_row {
    ($DB:ident, $R:ident) => {
        impl<O: Unpin, F> crate::query::MapRow<$DB> for F
        where
            F: Send + FnMut($R) -> O,
        {
            type Output = O;

            fn map_row(&mut self, row: $R) -> O {
                (self)(row)
            }
        }

        impl<O: Unpin, F> crate::query::TryMapRow<$DB> for F
        where
            F: Send + FnMut($R) -> Result<O, crate::error::Error>,
        {
            type Output = O;

            fn try_map_row(&mut self, row: $R) -> Result<O, crate::error::Error> {
                (self)(row)
            }
        }
    };
}
