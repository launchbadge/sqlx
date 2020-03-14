use std::collections::HashMap;
use std::sync::Arc;

use futures_core::future::BoxFuture;

use crate::connection::ConnectionSource;
use crate::cursor::Cursor;
use crate::executor::Execute;
use crate::maybe_owned::MaybeOwned;
use crate::pool::Pool;
use crate::sqlite::statement::{SqliteStatement, Step};
use crate::sqlite::{Sqlite, SqliteArguments, SqliteConnection, SqliteRow};

enum State<'q> {
    Query((&'q str, Option<SqliteArguments>)),
    Statement {
        query: &'q str,
        arguments: Option<SqliteArguments>,
        statement: MaybeOwned<SqliteStatement, usize>,
    },
}

pub struct SqliteCursor<'c, 'q> {
    source: ConnectionSource<'c, SqliteConnection>,
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
                let statement_ = statement.resolve(&mut conn.statements);

                if let Some(arguments) = arguments {
                    statement_.bind(arguments)?;
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
        }
    };

    let statement_ = statement.resolve(&mut conn.statements);

    match statement_.step().await? {
        Step::Done => {
            // TODO: If there is more to do, we need to do more
            Ok(None)
        }

        Step::Row => Ok(Some(SqliteRow {
            statement: &*statement_,
        })),
    }
}
