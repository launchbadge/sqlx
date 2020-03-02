use std::future::Future;
use std::marker::PhantomData;
use std::mem;
use std::pin::Pin;
use std::task::{Context, Poll};

use async_stream::try_stream;
use futures_core::stream::BoxStream;
use futures_core::Stream;
use futures_util::future::ready;
use futures_util::ready;
use futures_util::stream::try_unfold;
use futures_util::TryFutureExt;
use futures_util::TryStreamExt;

use crate::arguments::Arguments;
use crate::cursor::Cursor;
use crate::database::{Database, HasCursor, HasRow};
use crate::encode::Encode;
use crate::executor::{Execute, Executor, RefExecutor};
use crate::types::Type;
use crate::{Error, FromRow};
use futures_core::future::BoxFuture;

/// Raw SQL query with bind parameters. Returned by [`query`][crate::query::query].
pub struct Query<'q, DB, A = <DB as Database>::Arguments>
where
    DB: Database,
{
    pub(crate) query: &'q str,
    pub(crate) arguments: A,
    database: PhantomData<DB>,
}

/// SQL query that will map its results to owned Rust types.
///
/// Returned by [Query::map], `query!()`, etc. Has most of the same methods as [Query] but
/// the return types are changed to reflect the mapping. However, there is no equivalent of
/// [Query::execute] as it doesn't make sense to map the result type and then ignore it.
pub struct Map<'q, DB, F, A = <DB as Database>::Arguments>
where
    DB: Database,
{
    query: Query<'q, DB, A>,
    mapper: F,
}

#[doc(hidden)]
pub struct ImmutableArguments<DB: Database>(DB::Arguments);

// necessary because we can't have a blanket impl for `Query<'q, DB>`
// the compiler thinks that `ImmutableArguments<DB>` could be `DB::Arguments` even though
// that would be an infinitely recursive type
impl<'q, DB> Execute<'q, DB> for Query<'q, DB, ImmutableArguments<DB>>
where
    DB: Database,
{
    fn into_parts(self) -> (&'q str, Option<<DB as Database>::Arguments>) {
        (self.query, Some(self.arguments.0))
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
    pub fn bind_all(self, arguments: DB::Arguments) -> Query<'q, DB, ImmutableArguments<DB>> {
        Query {
            query: self.query,
            arguments: ImmutableArguments(arguments),
            database: PhantomData,
        }
    }
}

impl<'q, DB, A> Query<'q, DB, A>
where
    DB: Database,
{
    /// Map each row in the result to another type.
    ///
    /// The returned type has most of the same methods but does not have [`.execute()`][Query::execute].
    ///
    /// Stylistically, we recommend placing this call after any [`.bind()`][Query::bind]
    /// calls, just before [`.fetch()`][Query::fetch], etc.
    ///
    /// See also: [query_as].
    pub fn map<F>(self, mapper: F) -> Map<'q, DB, F, A>
    where
        F: MapRow<DB>,
    {
        Map {
            query: self,
            mapper,
        }
    }
}

impl<'q, DB, A> Query<'q, DB, A>
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

impl<'q, DB, F, A> Map<'q, DB, F, A>
where
    DB: Database,
    Query<'q, DB, A>: Execute<'q, DB>,
    F: MapRow<DB>,
{
    /// Execute the query and get a [Stream] of the results, returning our mapped type.
    pub fn fetch<'e: 'q, E>(
        mut self,
        executor: E,
    ) -> impl Stream<Item = crate::Result<F::Mapped>> + 'e
    where
        'q: 'e,
        E: RefExecutor<'e, Database = DB> + 'e,
        F: 'e,
        F::Mapped: 'e,
        A: 'e,
    {
        try_stream! {
            let mut cursor = executor.fetch_by_ref(self.query);
            while let Some(next) = cursor.next().await? {
                let mapped = self.mapper.map_row(next)?;
                yield mapped;
            }
        }
    }

    /// Get the first row in the result
    pub async fn fetch_optional<'e, E>(mut self, executor: E) -> crate::Result<Option<F::Mapped>>
    where
        E: RefExecutor<'e, Database = DB>,
        'q: 'e,
    {
        // could be implemented in terms of `fetch()` but this avoids overhead from `try_stream!`
        let mut cursor = executor.fetch_by_ref(self.query);
        let mut mapper = self.mapper;
        let val = cursor.next().await?;
        val.map(|row| mapper.map_row(row)).transpose()
    }

    pub async fn fetch_one<'e, E>(self, executor: E) -> crate::Result<F::Mapped>
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

    pub async fn fetch_all<'e, E>(mut self, executor: E) -> crate::Result<Vec<F::Mapped>>
    where
        E: RefExecutor<'e, Database = DB>,
        'q: 'e,
    {
        let mut cursor = executor.fetch_by_ref(self.query);
        let mut out = vec![];

        while let Some(row) = cursor.next().await? {
            out.push(self.mapper.map_row(row)?);
        }

        Ok(out)
    }
}

/// A (hopefully) temporary workaround for an internal compiler error (ICE) involving higher-ranked
/// trait bounds (HRTBs), associated types and closures.
///
/// See https://github.com/rust-lang/rust/issues/62529
pub trait MapRow<DB: Database> {
    type Mapped: Unpin;

    fn map_row(&mut self, row: <DB as HasRow>::Row) -> crate::Result<Self::Mapped>;
}

impl<O: Unpin, DB> MapRow<DB> for for<'c> fn(<DB as HasRow<'c>>::Row) -> crate::Result<O>
where
    DB: Database,
{
    type Mapped = O;

    fn map_row(&mut self, row: <DB as HasRow>::Row) -> crate::Result<O> {
        (self)(row)
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
