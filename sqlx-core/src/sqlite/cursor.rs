use core::mem::take;

use std::collections::HashMap;
use std::sync::Arc;

use futures_core::future::BoxFuture;

use crate::connection::ConnectionSource;
use crate::cursor::Cursor;
use crate::executor::Execute;
use crate::pool::Pool;
use crate::sqlite::statement::{SqliteStatement, Step};
use crate::sqlite::{Sqlite, SqliteArguments, SqliteConnection, SqliteRow};

enum State<'q> {
    Empty,
    Query((&'q str, Option<SqliteArguments>)),
    Statement {
        query: &'q str,
        arguments: Option<SqliteArguments>,
        statement: SqliteStatement,
    },
}

impl Default for State<'_> {
    fn default() -> Self {
        State::Empty
    }
}

pub struct SqliteCursor<'c, 'q> {
    source: ConnectionSource<'c, SqliteConnection>,
    // query: Option<(&'q str, Option<SqliteArguments>)>,
    columns: Arc<HashMap<Box<str>, usize>>,
    state: State<'q>,
}

impl<'c, 'q> Cursor<'c, 'q> for SqliteCursor<'c, 'q> {
    type Database = Sqlite;

    fn from_pool<E>(pool: &Pool<SqliteConnection>, query: E) -> Self
    where
        Self: Sized,
        E: Execute<'q, Sqlite>,
    {
        Self {
            source: ConnectionSource::Pool(pool.clone()),
            columns: Arc::default(),
            state: State::Query(query.into_parts()),
        }
    }

    fn from_connection<E>(conn: &'c mut SqliteConnection, query: E) -> Self
    where
        Self: Sized,
        E: Execute<'q, Sqlite>,
    {
        Self {
            source: ConnectionSource::Connection(conn.into()),
            columns: Arc::default(),
            state: State::Query(query.into_parts()),
        }
    }

    fn next(&mut self) -> BoxFuture<crate::Result<Option<SqliteRow<'_>>>> {
        Box::pin(next(self))
    }
}

async fn next<'a, 'c: 'a, 'q: 'a>(
    cursor: &'a mut SqliteCursor<'c, 'q>,
) -> crate::Result<Option<SqliteRow<'a>>> {
    let conn = cursor.source.resolve().await?;

    let statement = loop {
        match cursor.state {
            State::Query((query, ref mut arguments)) => {
                let mut statement = conn.prepare(query, arguments.is_some())?;

                if let Some(arguments) = arguments {
                    statement.bind(arguments)?;
                }

                cursor.state = State::Statement {
                    statement,
                    query,
                    arguments: arguments.take(),
                };
            }

            State::Statement {
                ref mut statement, ..
            } => {
                break statement;
            }

            State::Empty => unreachable!("use after drop"),
        }
    };

    match statement.step().await? {
        Step::Done => {
            // TODO: If there is more to do, we need to do more
            Ok(None)
        }

        Step::Row => Ok(Some(SqliteRow {
            statement,
            columns: Arc::default(),
        })),
    }
}

// If there is a statement on our WIP object
// Put it back into the cache IFF this is a persistent query
impl<'c, 'q> Drop for SqliteCursor<'c, 'q> {
    fn drop(&mut self) {
        match take(&mut self.state) {
            State::Statement {
                query,
                arguments,
                statement,
            } => {
                if arguments.is_some() {
                    if let ConnectionSource::Connection(connection) = &mut self.source {
                        connection
                            .cache_statement
                            .insert(query.to_owned(), statement);
                    }
                }
            }

            _ => {}
        }
    }
}
