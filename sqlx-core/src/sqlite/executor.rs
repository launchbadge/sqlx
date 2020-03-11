use futures_core::future::BoxFuture;

use crate::cursor::Cursor;
use crate::describe::Describe;
use crate::executor::{Execute, Executor, RefExecutor};
use crate::sqlite::cursor::SqliteCursor;
use crate::sqlite::Sqlite;

impl Executor for super::SqliteConnection {
    type Database = Sqlite;

    fn execute<'e, 'q, E: 'e>(&'e mut self, query: E) -> BoxFuture<'e, crate::Result<u64>>
    where
        E: Execute<'q, Self::Database>,
    {
        // Box::pin(async move {
        //     let (query, arguments) = query.into_parts();
        //
        //     self.run(query, arguments).await?;
        //     self.affected_rows().await
        // })
        todo!()
    }

    fn fetch<'q, E>(&mut self, query: E) -> SqliteCursor<'_, 'q>
    where
        E: Execute<'q, Self::Database>,
    {
        SqliteCursor::from_connection(self, query)
    }

    fn describe<'e, 'q, E: 'e>(
        &'e mut self,
        query: E,
    ) -> BoxFuture<'e, crate::Result<Describe<Self::Database>>>
    where
        E: Execute<'q, Self::Database>,
    {
        // Box::pin(async move { self.describe(query.into_parts().0).await })
        todo!()
    }
}

impl<'c> RefExecutor<'c> for &'c mut super::SqliteConnection {
    type Database = Sqlite;

    fn fetch_by_ref<'q, E>(self, query: E) -> SqliteCursor<'c, 'q>
    where
        E: Execute<'q, Self::Database>,
    {
        SqliteCursor::from_connection(self, query)
    }
}
