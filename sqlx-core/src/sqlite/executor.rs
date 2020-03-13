use futures_core::future::BoxFuture;

use libsqlite3_sys::sqlite3_changes;

use crate::cursor::Cursor;
use crate::describe::Describe;
use crate::executor::{Execute, Executor, RefExecutor};
use crate::sqlite::arguments::SqliteArguments;
use crate::sqlite::cursor::SqliteCursor;
use crate::sqlite::statement::{SqliteStatement, Step};
use crate::sqlite::{Sqlite, SqliteConnection};

impl SqliteConnection {
    pub(super) fn prepare(
        &mut self,
        query: &str,
        persistent: bool,
    ) -> crate::Result<SqliteStatement> {
        if let Some(mut statement) = self.cache_statement.remove(&*query) {
            // As this statement has very likely been used before, we reset
            // it to clear the bindings and its program state

            statement.reset();

            Ok(statement)
        } else {
            SqliteStatement::new(&mut self.handle, query, persistent)
        }
    }

    fn changes(&mut self) -> u64 {
        // https://www.sqlite.org/c3ref/changes.html
        #[allow(unsafe_code)]
        let changes = unsafe { sqlite3_changes(self.handle.as_ptr()) };
        changes as u64
    }
}

impl Executor for SqliteConnection {
    type Database = Sqlite;

    fn execute<'e, 'q: 'e, 'c: 'e, E: 'e>(
        &'c mut self,
        query: E,
    ) -> BoxFuture<'e, crate::Result<u64>>
    where
        E: Execute<'q, Self::Database>,
    {
        Box::pin(
            AffectedRows::<'c, 'q> {
                connection: self,
                query: query.into_parts(),
                statement: None,
            }
            .get(),
        )
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

impl<'e> RefExecutor<'e> for &'e mut SqliteConnection {
    type Database = Sqlite;

    fn fetch_by_ref<'q, E>(self, query: E) -> SqliteCursor<'e, 'q>
    where
        E: Execute<'q, Self::Database>,
    {
        SqliteCursor::from_connection(self, query)
    }
}

struct AffectedRows<'c, 'q> {
    query: (&'q str, Option<SqliteArguments>),
    connection: &'c mut SqliteConnection,
    statement: Option<SqliteStatement>,
}

impl AffectedRows<'_, '_> {
    async fn get(mut self) -> crate::Result<u64> {
        let mut statement = self
            .connection
            .prepare(self.query.0, self.query.1.is_some())?;

        if let Some(arguments) = &mut self.query.1 {
            statement.bind(arguments)?;
        }

        while let Step::Row = statement.step().await? {
            // we only care about the rows modified; ignore
        }

        Ok(self.connection.changes())
    }
}

impl Drop for AffectedRows<'_, '_> {
    fn drop(&mut self) {
        // If there is a statement on our WIP object
        // Put it back into the cache IFF this is a persistent query
        if self.query.1.is_some() {
            if let Some(statement) = self.statement.take() {
                self.connection
                    .cache_statement
                    .insert(self.query.0.to_owned(), statement);
            }
        }
    }
}
