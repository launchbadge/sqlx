use core::marker::PhantomData;

use crate::arguments::Arguments;
use crate::database::Database;
use crate::encode::Encode;
use crate::executor::Execute;
use crate::types::Type;

/// Raw SQL query with bind parameters, mapped to a concrete type
/// using [`FromRow`](trait.FromRow.html). Returned
/// by [`query_as`](fn.query_as.html).
pub struct QueryAs<'q, DB, O>
where
    DB: Database,
{
    query: &'q str,
    arguments: <DB as Database>::Arguments,
    database: PhantomData<DB>,
    output: PhantomData<O>,
}

impl<'q, DB, O> QueryAs<'q, DB, O>
where
    DB: Database,
{
    /// Bind a value for use with this SQL query.
    #[inline]
    pub fn bind<T>(mut self, value: T) -> Self
    where
        T: Type<DB>,
        T: Encode<DB>,
    {
        self.arguments.add(value);
        self
    }
}

impl<'q, DB, O: Send> Execute<'q, DB> for QueryAs<'q, DB, O>
where
    DB: Database,
{
    #[inline]
    fn into_parts(self) -> (&'q str, Option<<DB as Database>::Arguments>) {
        (self.query, Some(self.arguments))
    }
}

/// Construct a raw SQL query that is mapped to a concrete type
/// using [`FromRow`](crate::row::FromRow).
///
/// Returns [`QueryAs`].
pub fn query_as<DB, O>(sql: &str) -> QueryAs<DB, O>
where
    DB: Database,
{
    QueryAs {
        query: sql,
        arguments: Default::default(),
        database: PhantomData,
        output: PhantomData,
    }
}

// We need database-specific QueryAs traits to work around:
//  https://github.com/rust-lang/rust/issues/62529

// If for some reason we miss that issue being resolved in a _stable_ edition of
// rust, please open up a 100 issues and shout as loud as you can to remove
// this unseemly hack.

#[allow(unused_macros)]
macro_rules! make_query_as {
    ($name:ident, $db:ident, $row:ident) => {
        pub trait $name<'q, O> {
            fn fetch<'e, E>(
                self,
                executor: E,
            ) -> futures_core::stream::BoxStream<'e, crate::Result<$db, O>>
            where
                E: 'e + Send + crate::executor::RefExecutor<'e, Database = $db>,
                O: 'e + Send + Unpin + for<'c> crate::row::FromRow<'c, $row<'c>>,
                'q: 'e;

            fn fetch_all<'e, E>(
                self,
                executor: E,
            ) -> futures_core::future::BoxFuture<'e, crate::Result<$db, Vec<O>>>
            where
                E: 'e + Send + crate::executor::RefExecutor<'e, Database = $db>,
                O: 'e + Send + for<'c> crate::row::FromRow<'c, $row<'c>>,
                'q: 'e;

            fn fetch_one<'e, E>(
                self,
                executor: E,
            ) -> futures_core::future::BoxFuture<'e, crate::Result<$db, O>>
            where
                E: 'e + Send + crate::executor::RefExecutor<'e, Database = $db>,
                O: 'e + Send + for<'c> crate::row::FromRow<'c, $row<'c>>,
                'q: 'e;

            fn fetch_optional<'e, E>(
                self,
                executor: E,
            ) -> futures_core::future::BoxFuture<'e, crate::Result<$db, Option<O>>>
            where
                E: 'e + Send + crate::executor::RefExecutor<'e, Database = $db>,
                O: 'e + Send + for<'c> crate::row::FromRow<'c, $row<'c>>,
                'q: 'e;
        }

        impl<'q, O> $name<'q, O> for crate::query_as::QueryAs<'q, $db, O> {
            fn fetch<'e, E>(
                self,
                executor: E,
            ) -> futures_core::stream::BoxStream<'e, crate::Result<$db, O>>
            where
                E: 'e + Send + crate::executor::RefExecutor<'e, Database = $db>,
                O: 'e + Send + Unpin + for<'c> crate::row::FromRow<'c, $row<'c>>,
                'q: 'e,
            {
                use crate::cursor::Cursor;

                Box::pin(async_stream::try_stream! {
                    let mut cursor = executor.fetch_by_ref(self);

                    while let Some(row) = cursor.next().await? {
                        let obj = O::from_row(row)?;

                        yield obj;
                    }
                })
            }

            fn fetch_optional<'e, E>(
                self,
                executor: E,
            ) -> futures_core::future::BoxFuture<'e, crate::Result<$db, Option<O>>>
            where
                E: 'e + Send + crate::executor::RefExecutor<'e, Database = $db>,
                O: 'e + Send + for<'c> crate::row::FromRow<'c, $row<'c>>,
                'q: 'e,
            {
                use crate::cursor::Cursor;

                Box::pin(async move {
                    let mut cursor = executor.fetch_by_ref(self);
                    let row = cursor.next().await?;

                    row.map(O::from_row).transpose()
                })
            }

            fn fetch_one<'e, E>(
                self,
                executor: E,
            ) -> futures_core::future::BoxFuture<'e, crate::Result<$db, O>>
            where
                E: 'e + Send + crate::executor::RefExecutor<'e, Database = $db>,
                O: 'e + Send + for<'c> crate::row::FromRow<'c, $row<'c>>,
                'q: 'e,
            {
                use futures_util::TryFutureExt;

                Box::pin(self.fetch_optional(executor).and_then(|row| match row {
                    Some(row) => futures_util::future::ready(Ok(row)),
                    None => futures_util::future::ready(Err(crate::Error::RowNotFound)),
                }))
            }

            fn fetch_all<'e, E>(
                self,
                executor: E,
            ) -> futures_core::future::BoxFuture<'e, crate::Result<$db, Vec<O>>>
            where
                E: 'e + Send + crate::executor::RefExecutor<'e, Database = $db>,
                O: 'e + Send + for<'c> crate::row::FromRow<'c, $row<'c>>,
                'q: 'e,
            {
                use crate::cursor::Cursor;

                Box::pin(async move {
                    let mut cursor = executor.fetch_by_ref(self);
                    let mut out = Vec::new();

                    while let Some(row) = cursor.next().await? {
                        let obj = O::from_row(row)?;

                        out.push(obj);
                    }

                    Ok(out)
                })
            }
        }
    };
}
