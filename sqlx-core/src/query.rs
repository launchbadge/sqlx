use crate::arguments::Arguments;
use crate::arguments::IntoArguments;
use crate::database::Database;
use crate::encode::Encode;
use crate::executor::Executor;
use crate::row::FromRow;
use crate::types::HasSqlType;
use futures_core::future::BoxFuture;
use futures_core::stream::BoxStream;
use futures_util::TryFutureExt;
use futures_util::TryStreamExt;
use std::marker::PhantomData;

/// A SQL query with bind parameters and output type.
///
/// Optionally type-safe if constructed through [query!].
pub struct Query<'q, DB, T = <DB as Database>::Arguments, R = <DB as Database>::Row>
where
    DB: Database,
{
    query: &'q str,
    arguments: T,
    record: PhantomData<R>,
    database: PhantomData<DB>,
}

impl<'q, DB, P: 'q, R: 'q> Query<'q, DB, P, R>
where
    DB: Database,
    DB::Arguments: 'q,
    P: IntoArguments<DB> + Send,
    R: FromRow<DB::Row> + Send + Unpin,
{
    pub fn execute<'e, E>(self, executor: &'e mut E) -> BoxFuture<'e, crate::Result<u64>>
    where
        E: Executor<Database = DB>,
        'q: 'e,
    {
        executor.execute(self.query, self.arguments.into_arguments())
    }

    pub fn fetch<'e, E>(self, executor: &'e mut E) -> BoxStream<'e, crate::Result<R>>
    where
        E: Executor<Database = DB>,
        DB::Row: 'e,
        'q: 'e,
    {
        Box::pin(
            executor
                .fetch(self.query, self.arguments.into_arguments())
                .map_ok(FromRow::from_row),
        )
    }

    pub fn fetch_all<'e: 'q, E>(self, executor: &'e mut E) -> BoxFuture<'e, crate::Result<Vec<R>>>
    where
        E: Executor<Database = DB>,
        DB::Row: 'e,
        'q: 'e,
    {
        Box::pin(self.fetch(executor).try_collect())
    }

    pub fn fetch_optional<'e: 'q, E>(
        self,
        executor: &'e mut E,
    ) -> BoxFuture<'e, crate::Result<Option<R>>>
    where
        E: Executor<Database = DB>,
        DB::Row: 'e,
        'q: 'e,
    {
        Box::pin(
            executor
                .fetch_optional(self.query, self.arguments.into_arguments())
                .map_ok(|row| row.map(FromRow::from_row)),
        )
    }

    pub fn fetch_one<'e: 'q, E>(self, executor: &'e mut E) -> BoxFuture<'e, crate::Result<R>>
    where
        E: Executor<Database = DB>,
        DB::Row: 'e,
        'q: 'e,
    {
        Box::pin(
            executor
                .fetch_one(self.query, self.arguments.into_arguments())
                .map_ok(FromRow::from_row),
        )
    }
}

impl<'q, DB> Query<'q, DB>
where
    DB: Database,
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
        self.arguments.add(value);
        self
    }
}

/// Construct a full SQL query that can be chained to bind parameters and executed.
///
/// # Examples
///
/// ```ignore
/// let names: Vec<String> = sqlx::query("SELECT name FROM users WHERE active = ?")
///     .bind(false) // [active = ?]
///     .fetch(&mut connection) // -> Stream<Item = impl Row>
///     .map_ok(|row| row.name("name")) // -> Stream<Item = String>
///     .try_collect().await?; // -> Vec<String>
/// ```
pub fn query<'q, DB>(sql: &'q str) -> Query<'q, DB>
where
    DB: Database,
{
    Query {
        database: PhantomData,
        record: PhantomData,
        arguments: Default::default(),
        query: sql.as_ref(),
    }
}
