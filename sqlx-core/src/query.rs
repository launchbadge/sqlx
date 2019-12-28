use crate::arguments::Arguments;
use crate::arguments::IntoArguments;
use crate::database::Database;
use crate::encode::Encode;
use crate::executor::Executor;
use crate::types::HasSqlType;
use futures_core::stream::BoxStream;
use futures_util::TryStreamExt;
use std::marker::PhantomData;

/// Dynamic SQL query with bind parameters. Returned by [query].
pub struct Query<'q, DB, T = <DB as Database>::Arguments>
where
    DB: Database,
{
    query: &'q str,
    arguments: T,
    database: PhantomData<DB>,
}

impl<'q, DB, P> Query<'q, DB, P>
where
    DB: Database,
    P: IntoArguments<DB> + Send,
{
    pub async fn execute<E>(self, executor: &mut E) -> crate::Result<u64>
    where
        E: Executor<Database = DB>,
    {
        executor
            .execute(self.query, self.arguments.into_arguments())
            .await
    }

    pub fn fetch<'e, E>(self, executor: &'e mut E) -> BoxStream<'e, crate::Result<DB::Row>>
    where
        E: Executor<Database = DB>,
        'q: 'e,
    {
        executor.fetch(self.query, self.arguments.into_arguments())
    }

    pub async fn fetch_all<E>(self, executor: &mut E) -> crate::Result<Vec<DB::Row>>
    where
        E: Executor<Database = DB>,
    {
        executor
            .fetch(self.query, self.arguments.into_arguments())
            .try_collect()
            .await
    }

    pub async fn fetch_optional<E>(self, executor: &mut E) -> crate::Result<Option<DB::Row>>
    where
        E: Executor<Database = DB>,
    {
        executor
            .fetch_optional(self.query, self.arguments.into_arguments())
            .await
    }

    pub async fn fetch_one<E>(self, executor: &mut E) -> crate::Result<DB::Row>
    where
        E: Executor<Database = DB>,
    {
        executor
            .fetch_one(self.query, self.arguments.into_arguments())
            .await
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
