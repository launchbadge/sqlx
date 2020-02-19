use crate::arguments::Arguments;
use crate::arguments::IntoArguments;
use crate::cursor::Cursor;
use crate::database::{Database, HasCursor, HasRow};
use crate::encode::Encode;
use crate::executor::{Execute, Executor};
use crate::types::HasSqlType;
use futures_core::stream::BoxStream;
use futures_util::future::ready;
use futures_util::TryFutureExt;
use futures_util::TryStreamExt;
use std::future::Future;
use std::marker::PhantomData;

/// Raw SQL query with bind parameters. Returned by [`query`].
pub struct Query<'a, DB, T = <DB as Database>::Arguments>
where
    DB: Database,
{
    query: &'a str,
    arguments: T,
    database: PhantomData<DB>,
}

impl<'a, DB, P> Execute<'a, DB> for Query<'a, DB, P>
where
    DB: Database,
    P: IntoArguments<DB> + Send,
{
    fn into_parts(self) -> (&'a str, Option<<DB as Database>::Arguments>) {
        (self.query, Some(self.arguments.into_arguments()))
    }
}

impl<'a, DB, P> Query<'a, DB, P>
where
    DB: Database,
    P: IntoArguments<DB> + Send,
{
    pub fn execute<'b, E>(self, executor: E) -> impl Future<Output = crate::Result<u64>> + 'b
    where
        E: Executor<'b, Database = DB>,
        'a: 'b,
    {
        executor.execute(self)
    }

    pub fn fetch<'b, E>(self, executor: E) -> <DB as HasCursor<'b>>::Cursor
    where
        E: Executor<'b, Database = DB>,
        'a: 'b,
    {
        executor.execute(self)
    }

    pub async fn fetch_optional<'b, E>(
        self,
        executor: E,
    ) -> crate::Result<Option<<DB as HasRow>::Row>>
    where
        E: Executor<'b, Database = DB>,
    {
        executor.execute(self).first().await
    }

    pub async fn fetch_one<'b, E>(self, executor: E) -> crate::Result<<DB as HasRow>::Row>
    where
        E: Executor<'b, Database = DB>,
    {
        self.fetch_optional(executor)
            .and_then(|row| match row {
                Some(row) => ready(Ok(row)),
                None => ready(Err(crate::Error::NotFound)),
            })
            .await
    }
}

impl<'q, DB> Query<'q, DB>
where
    DB: Database,
{
    /// Bind a value for use with this SQL query.
    pub fn bind<T>(mut self, value: T) -> Self
    where
        DB: HasSqlType<T>,
        T: Encode<DB>,
    {
        self.arguments.add(value);
        self
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
