use futures_core::future::BoxFuture;

use libsqlite3_sys::sqlite3_changes;

use crate::cursor::Cursor;
use crate::describe::{Column, Describe};
use crate::executor::{Execute, Executor, RefExecutor};
use crate::sqlite::cursor::SqliteCursor;
use crate::sqlite::statement::{Statement, Step};
use crate::sqlite::type_info::SqliteType;
use crate::sqlite::{Sqlite, SqliteConnection, SqliteTypeInfo};

impl SqliteConnection {
    pub(super) fn prepare(
        &mut self,
        query: &mut &str,
        persistent: bool,
    ) -> crate::Result<Option<usize>> {
        // TODO: Revisit statement caching and allow cache expiration by using a
        //       generational index

        if !persistent {
            // A non-persistent query will be immediately prepared and returned,
            // regardless of the current state of the cache
            self.statement = Some(Statement::new(self, query, false)?);
            return Ok(None);
        }

        if let Some(key) = self.statement_by_query.get(&**query) {
            let statement = &mut self.statements[*key];

            // Adjust the passed in query string as if [string3_prepare]
            // did the tail parsing
            *query = &query[statement.tail..];

            // As this statement has very likely been used before, we reset
            // it to clear the bindings and its program state
            statement.reset();

            return Ok(Some(*key));
        }

        // Prepare a new statement object; ensuring to tell SQLite that this will be stored
        // for a "long" time and re-used multiple times

        let query_key = query.to_owned();
        let statement = Statement::new(self, query, true)?;

        let key = self.statements.len();

        self.statement_by_query.insert(query_key, key);
        self.statements.push(statement);

        Ok(Some(key))
    }

    // This is used for [affected_rows] in the public API.
    fn changes(&mut self) -> u64 {
        // Returns the number of rows modified, inserted or deleted by the most recently
        // completed INSERT, UPDATE or DELETE statement.

        // https://www.sqlite.org/c3ref/changes.html
        let changes = unsafe { sqlite3_changes(self.handle()) };
        changes as u64
    }

    #[inline]
    pub(super) fn statement(&self, key: Option<usize>) -> &Statement {
        match key {
            Some(key) => &self.statements[key],
            None => self.statement.as_ref().unwrap(),
        }
    }

    #[inline]
    pub(super) fn statement_mut(&mut self, key: Option<usize>) -> &mut Statement {
        match key {
            Some(key) => &mut self.statements[key],
            None => self.statement.as_mut().unwrap(),
        }
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
            loop {
                let key = self.prepare(&mut query, arguments.is_some())?;
                let statement = self.statement_mut(key);

                if let Some(arguments) = &mut arguments {
                    statement.bind(arguments)?;
                }

                while let Step::Row = statement.step().await? {
                    // We only care about the rows modified; ignore
                }

                if query.is_empty() {
                    break;
                }
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

    #[doc(hidden)]
    fn describe<'e, 'q, E: 'e>(
        &'e mut self,
        query: E,
    ) -> BoxFuture<'e, crate::Result<Describe<Self::Database>>>
    where
        E: Execute<'q, Self::Database>,
    {
        Box::pin(async move {
            let (mut query, _) = query.into_parts();
            let key = self.prepare(&mut query, false)?;
            let statement = self.statement_mut(key);

            // First let's attempt to describe what we can about parameter types
            // Which happens to just be the count, heh
            let num_params = statement.params();
            let params = vec![None; num_params].into_boxed_slice();

            // Next, collect (return) column types and names
            let num_columns = statement.column_count();
            let mut columns = Vec::with_capacity(num_columns);
            for i in 0..num_columns {
                let name = statement.column_name(i).to_owned();
                let decl = statement.column_decltype(i);

                let r#type = match decl {
                    None => None,
                    Some(decl) => match &*decl.to_ascii_lowercase() {
                        "bool" | "boolean" => Some(SqliteType::Boolean),
                        "clob" | "text" => Some(SqliteType::Text),
                        "blob" => Some(SqliteType::Blob),
                        "real" | "double" | "double precision" | "float" => Some(SqliteType::Float),
                        decl @ _ if decl.contains("int") => Some(SqliteType::Integer),
                        decl @ _ if decl.contains("char") => Some(SqliteType::Text),
                        _ => None,
                    },
                };

                columns.push(Column {
                    name: Some(name.into()),
                    non_null: statement.column_not_null(i)?,
                    table_id: None,
                    type_info: r#type.map(|r#type| SqliteTypeInfo {
                        r#type,
                        affinity: None,
                    }),
                })
            }

            Ok(Describe {
                param_types: params,
                result_columns: columns.into_boxed_slice(),
            })
        })
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
