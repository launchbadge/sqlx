use crate::arguments::Arguments;
use crate::arguments::IntoArguments;
use crate::cursor::Cursor;
use crate::database::{Database, HasCursor, HasRow};
use crate::encode::Encode;
use crate::executor::{Execute, Executor};
use crate::types::Type;
use futures_core::stream::BoxStream;
use futures_util::future::ready;
use futures_util::TryFutureExt;
use futures_util::TryStreamExt;
use std::future::Future;
use std::marker::PhantomData;
use std::mem;

/// Raw SQL query with bind parameters. Returned by [`query`].
pub struct Query<'q, DB, T = <DB as Database>::Arguments>
where
    DB: Database,
{
    query: &'q str,
    arguments: T,
    database: PhantomData<DB>,
}

impl<'q, DB, P> Execute<'q, DB> for Query<'q, DB, P>
where
    DB: Database,
    P: IntoArguments<DB> + Send,
{
    fn into_parts(self) -> (&'q str, Option<<DB as Database>::Arguments>) {
        (self.query, Some(self.arguments.into_arguments()))
    }
}

impl<'q, DB, P> Query<'q, DB, P>
where
    DB: Database,
    P: IntoArguments<DB> + Send,
{
    pub async fn execute<'e, E>(self, executor: E) -> crate::Result<u64>
    where
        E: Executor<'e, Database = DB>,
    {
        executor.execute(self).await
    }

    pub fn fetch<'e, E>(self, executor: E) -> <DB as HasCursor<'e, 'q, DB>>::Cursor
    where
        E: Executor<'e, Database = DB>,
    {
        executor.execute(self)
    }

    pub async fn fetch_optional<'e, E>(
        self,
        executor: E,
    ) -> crate::Result<Option<<DB as HasRow<'e>>::Row>>
    where
        E: Executor<'e, Database = DB>,
        'q: 'e,
    {
        executor.execute(self).first().await
    }

    pub async fn fetch_one<'e, E>(self, executor: E) -> crate::Result<<DB as HasRow<'e>>::Row>
    where
        E: Executor<'e, Database = DB>,
        'q: 'e,
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
        T: Type<DB>,
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
