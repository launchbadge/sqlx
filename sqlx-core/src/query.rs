use std::marker::PhantomData;

use async_stream::try_stream;
use futures_core::Stream;
use futures_util::future::ready;
use futures_util::TryFutureExt;

use crate::arguments::Arguments;
use crate::cursor::{Cursor, HasCursor};
use crate::database::Database;
use crate::encode::Encode;
use crate::executor::{Execute, Executor, RefExecutor};
use crate::row::HasRow;
use crate::types::Type;

/// Raw SQL query with bind parameters. Returned by [`query`][crate::query::query].
pub struct Query<'q, DB>
where
    DB: Database,
{
    pub(crate) query: &'q str,
    pub(crate) arguments: DB::Arguments,
    database: PhantomData<DB>,
}

/// SQL query that will map its results to owned Rust types.
///
/// Returned by [Query::try_map], `query!()`, etc. Has most of the same methods as [Query] but
/// the return types are changed to reflect the mapping. However, there is no equivalent of
/// [Query::execute] as it doesn't make sense to map the result type and then ignore it.
///
/// [Query::bind] is also omitted; stylistically we recommend placing your `.bind()` calls
/// before `.try_map()` anyway.
pub struct Map<'q, DB, F>
where
    DB: Database,
{
    query: Query<'q, DB>,
    mapper: F,
}

// necessary because we can't have a blanket impl for `Query<'q, DB>`
// the compiler thinks that `ImmutableArguments<DB>` could be `DB::Arguments` even though
// that would be an infinitely recursive type
impl<'q, DB> Execute<'q, DB> for Query<'q, DB>
where
    DB: Database,
{
    fn into_parts(self) -> (&'q str, Option<DB::Arguments>) {
        (self.query, Some(self.arguments))
    }
}

impl<'q, DB> Query<'q, DB>
where
    DB: Database,
{
    /// Bind a value for use with this SQL query.
    ///
    /// If the number of times this is called does not match the number of bind parameters that
    /// appear in the query (`?` for most SQL flavors, `$1 .. $N` for Postgres) then an error
    /// will be returned when this query is executed.
    ///
    /// There is no validation that the value is of the type expected by the query. Most SQL
    /// flavors will perform type coercion (Postgres will return a database error).s
    pub fn bind<T>(mut self, value: T) -> Self
    where
        T: Type<DB>,
        T: Encode<DB>,
    {
        self.arguments.add(value);
        self
    }

    #[doc(hidden)]
    pub fn bind_all(self, arguments: DB::Arguments) -> Query<'q, DB> {
        Query {
            query: self.query,
            arguments,
            database: PhantomData,
        }
    }
}

impl<'q, DB> Query<'q, DB>
where
    DB: Database,
{
    /// Map each row in the result to another type.
    ///
    /// The returned type has most of the same methods but does not have
    /// [`.execute()`][Query::execute] or [`.bind()][Query::bind].
    ///
    /// See also: [query_as][crate::query_as::query_as].
    pub fn map<F, O>(self, mapper: F) -> Map<'q, DB, impl TryMapRow<DB, Output = O>>
    where
        O: Unpin,
        F: MapRow<DB, Output = O>,
    {
        self.try_map(MapRowAdapter(mapper))
    }

    /// Map each row in the result to another type.
    ///
    /// See also: [query_as][crate::query_as::query_as].
    pub fn try_map<F>(self, mapper: F) -> Map<'q, DB, F>
    where
        F: TryMapRow<DB>,
    {
        Map {
            query: self,
            mapper,
        }
    }
}

impl<'q, DB> Query<'q, DB>
where
    DB: Database,
    Self: Execute<'q, DB>,
{
    pub async fn execute<E>(self, mut executor: E) -> crate::Result<u64>
    where
        E: Executor<Database = DB>,
    {
        executor.execute(self).await
    }

    pub fn fetch<'e, E>(self, executor: E) -> <DB as HasCursor<'e, 'q>>::Cursor
    where
        E: RefExecutor<'e, Database = DB>,
    {
        executor.fetch_by_ref(self)
    }
}

impl<'q, DB, F> Map<'q, DB, F>
where
    DB: Database,
    Query<'q, DB>: Execute<'q, DB>,
    F: TryMapRow<DB>,
{
    /// Execute the query and get a [Stream] of the results, returning our mapped type.
    pub fn fetch<'e: 'q, E>(
        mut self,
        executor: E,
    ) -> impl Stream<Item = crate::Result<F::Output>> + 'e
    where
        'q: 'e,
        E: RefExecutor<'e, Database = DB> + 'e,
        F: 'e,
        F::Output: 'e,
    {
        try_stream! {
            let mut cursor = executor.fetch_by_ref(self.query);
            while let Some(next) = cursor.next().await? {
                let mapped = self.mapper.try_map_row(next)?;
                yield mapped;
            }
        }
    }

    /// Get the first row in the result
    pub async fn fetch_optional<'e, E>(self, executor: E) -> crate::Result<Option<F::Output>>
    where
        E: RefExecutor<'e, Database = DB>,
        'q: 'e,
    {
        // could be implemented in terms of `fetch()` but this avoids overhead from `try_stream!`
        let mut cursor = executor.fetch_by_ref(self.query);
        let mut mapper = self.mapper;
        let val = cursor.next().await?;
        val.map(|row| mapper.try_map_row(row)).transpose()
    }

    pub async fn fetch_one<'e, E>(self, executor: E) -> crate::Result<F::Output>
    where
        E: RefExecutor<'e, Database = DB>,
        'q: 'e,
    {
        self.fetch_optional(executor)
            .and_then(|row| match row {
                Some(row) => ready(Ok(row)),
                None => ready(Err(crate::Error::RowNotFound)),
            })
            .await
    }

    pub async fn fetch_all<'e, E>(mut self, executor: E) -> crate::Result<Vec<F::Output>>
    where
        E: RefExecutor<'e, Database = DB>,
        'q: 'e,
    {
        let mut cursor = executor.fetch_by_ref(self.query);
        let mut out = vec![];

        while let Some(row) = cursor.next().await? {
            out.push(self.mapper.try_map_row(row)?);
        }

        Ok(out)
    }
}

// A (hopefully) temporary workaround for an internal compiler error (ICE) involving higher-ranked
// trait bounds (HRTBs), associated types and closures.
//
// See https://github.com/rust-lang/rust/issues/62529

pub trait TryMapRow<DB: Database> {
    type Output: Unpin;

    fn try_map_row(&mut self, row: <DB as HasRow>::Row) -> crate::Result<Self::Output>;
}

pub trait MapRow<DB: Database> {
    type Output: Unpin;

    fn map_row(&mut self, row: <DB as HasRow>::Row) -> Self::Output;
}

// An adapter that implements [MapRow] in terms of [TryMapRow]
// Just ends up Ok wrapping it

struct MapRowAdapter<F>(F);

impl<DB: Database, O, F> TryMapRow<DB> for MapRowAdapter<F>
where
    O: Unpin,
    F: MapRow<DB, Output = O>,
{
    type Output = O;

    fn try_map_row(&mut self, row: <DB as HasRow>::Row) -> crate::Result<Self::Output> {
        Ok(self.0.map_row(row))
    }
}

/// Construct a raw SQL query that can be chained to bind parameters and executed.
pub fn query<DB>(sql: &str) -> Query<DB>
where
    DB: Database,
{
    Query {
        database: PhantomData,
        arguments: Default::default(),
        query: sql,
    }
}
