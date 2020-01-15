use futures_core::Stream;
use futures_util::{future, TryStreamExt};

use crate::arguments::{Arguments, ImmutableArguments};
use crate::{
    arguments::IntoArguments, database::Database, encode::Encode, executor::Executor, row::FromRow,
    types::HasSqlType,
};

/// SQL query with bind parameters, which maps rows to an explicit output type.
///
/// Returned by [query_as] and [query!] *et al*.
///
/// The methods on this struct should be passed a reference to [crate::Pool] or one of
/// the connection types.
pub struct QueryAs<'q, DB, R, P = <DB as Database>::Arguments>
where
    DB: Database,
{
    query: &'q str,
    args: P,
    map_row: fn(DB::Row) -> crate::Result<R>,
}

/// The result of [query!] for SQL queries that does not return output.
impl<DB, P> QueryAs<'_, DB, (), P>
where
    DB: Database,
    P: IntoArguments<DB> + Send,
{
    /// Execute the query for its side-effects.
    ///
    /// Returns the number of rows affected, or 0 if not applicable.
    pub async fn execute<E>(self, executor: &mut E) -> crate::Result<u64>
    where
        E: Executor<Database = DB>,
    {
        executor
            .execute(self.query, self.args.into_arguments())
            .await
    }
}

impl<'q, DB, R, P> QueryAs<'q, DB, R, P>
where
    DB: Database,
    P: IntoArguments<DB> + Send,
    R: Send + 'q,
{
    /// Execute the query, returning the rows as a futures `Stream`.
    ///
    /// Use [fetch_all] if you want a `Vec` instead.
    pub fn fetch<'e, E>(self, executor: &'e mut E) -> impl Stream<Item = crate::Result<R>> + 'e
    where
        E: Executor<Database = DB>,
        'q: 'e,
    {
        let Self {
            query,
            args,
            map_row,
            ..
        } = self;
        executor
            .fetch(query, args.into_arguments())
            .and_then(move |row| future::ready(map_row(row)))
    }

    /// Execute the query and get all rows from the result as a `Vec`.
    pub async fn fetch_all<E>(self, executor: &mut E) -> crate::Result<Vec<R>>
    where
        E: Executor<Database = DB>,
    {
        self.fetch(executor).try_collect().await
    }

    /// Execute a query which should return either 0 or 1 rows.
    ///
    /// Returns [crate::Error::FoundMoreThanOne] if more than 1 row is returned.
    /// Use `.fetch().try_next()` if you just want one row.
    pub async fn fetch_optional<E>(self, executor: &mut E) -> crate::Result<Option<R>>
    where
        E: Executor<Database = DB>,
    {
        executor
            .fetch_optional(self.query, self.args.into_arguments())
            .await?
            .map(self.map_row)
            .transpose()
    }

    /// Execute a query which should return exactly 1 row.
    ///
    /// * Returns [crate::Error::NotFound] if 0 rows are returned.
    /// * Returns [crate::Error::FoundMoreThanOne] if more than one row is returned.
    pub async fn fetch_one<E>(self, executor: &mut E) -> crate::Result<R>
    where
        E: Executor<Database = DB>,
    {
        (self.map_row)(
            executor
                .fetch_one(self.query, self.args.into_arguments())
                .await?,
        )
    }
}

impl<'q, DB, R> QueryAs<'q, DB, R>
where
    DB: Database,
    DB::Arguments: Arguments<Database = DB>,
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
        self.args.add(value);
        self
    }

    // used by query!() and friends
    #[doc(hidden)]
    pub fn bind_all(self, values: DB::Arguments) -> QueryAs<'q, DB, R, ImmutableArguments<DB>> {
        QueryAs {
            query: self.query,
            args: ImmutableArguments(values),
            map_row: self.map_row,
        }
    }
}

/// Construct a dynamic SQL query with an explicit output type implementing [FromRow].
#[inline]
pub fn query_as<DB, T>(query: &str) -> QueryAs<DB, T>
where
    DB: Database,
    T: FromRow<DB::Row>,
{
    QueryAs {
        query,
        args: Default::default(),
        map_row: |row| Ok(T::from_row(row)),
    }
}

#[doc(hidden)]
pub fn query_as_mapped<DB, T>(
    query: &str,
    map_row: fn(DB::Row) -> crate::Result<T>,
) -> QueryAs<DB, T>
where
    DB: Database,
{
    QueryAs {
        query,
        args: Default::default(),
        map_row,
    }
}
