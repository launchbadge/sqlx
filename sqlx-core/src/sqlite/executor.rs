use futures_core::future::BoxFuture;

use libsqlite3_sys::sqlite3_changes;

use crate::cursor::Cursor;
use crate::describe::Describe;
use crate::executor::{Execute, Executor, RefExecutor};
use crate::maybe_owned::MaybeOwned;
use crate::sqlite::arguments::SqliteArguments;
use crate::sqlite::cursor::SqliteCursor;
use crate::sqlite::statement::{SqliteStatement, Step};
use crate::sqlite::{Sqlite, SqliteConnection};
use std::collections::HashMap;

impl SqliteConnection {
    pub(super) fn prepare(
        &mut self,
        query: &str,
        persistent: bool,
    ) -> crate::Result<MaybeOwned<SqliteStatement, usize>> {
        // TODO: Revisit statement caching and allow cache expiration by using a
        //       generational index

        if !persistent {
            // A non-persistent query will be immediately prepared and returned
            return SqliteStatement::new(&mut self.handle, query, false).map(MaybeOwned::Owned);
        }

        if let Some(key) = self.statement_by_query.get(query) {
            let statement = &mut self.statements[*key];

            // As this statement has very likely been used before, we reset
            // it to clear the bindings and its program state
            statement.reset();

            return Ok(MaybeOwned::Borrowed(*key));
        }

        // Prepare a new statement object; ensuring to tell SQLite that this will be stored
        // for a "long" time and re-used multiple times

        let key = self.statements.len();

        self.statement_by_query.insert(query.to_owned(), key);
        self.statements
            .push(SqliteStatement::new(&mut self.handle, query, true)?);

        Ok(MaybeOwned::Borrowed(key))
    }

    // This is used for [affected_rows] in the public API.
    fn changes(&mut self) -> u64 {
        // Returns the number of rows modified, inserted or deleted by the most recently
        // completed INSERT, UPDATE or DELETE statement.

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
        let (mut query, mut arguments) = query.into_parts();

        Box::pin(async move {
            let mut statement = self.prepare(query, arguments.is_some())?;
            let mut statement_ = statement.resolve(&mut self.statements);

            if let Some(arguments) = &mut arguments {
                statement_.bind(arguments)?;
            }

            while let Step::Row = statement_.step().await? {
                // We only care about the rows modified; ignore
            }

            Ok(self.changes())
        })
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
