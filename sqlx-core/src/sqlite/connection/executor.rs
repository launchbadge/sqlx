use std::sync::Arc;

use either::Either;
use futures_core::future::BoxFuture;
use futures_core::stream::BoxStream;
use futures_util::TryStreamExt;
use hashbrown::HashMap;

use crate::describe::{Column, Describe};
use crate::error::Error;
use crate::executor::{Execute, Executor};
use crate::ext::ustr::UStr;
use crate::sqlite::connection::ConnectionHandle;
use crate::sqlite::statement::{SqliteStatement, StatementHandle};
use crate::sqlite::{Sqlite, SqliteArguments, SqliteConnection, SqliteRow};

fn prepare<'a>(
    conn: &mut ConnectionHandle,
    statements: &'a mut HashMap<String, SqliteStatement>,
    statement: &'a mut Option<SqliteStatement>,
    query: &str,
    persistent: bool,
) -> Result<&'a mut SqliteStatement, Error> {
    if !persistent {
        *statement = Some(SqliteStatement::prepare(conn, query, false)?);
        return Ok(statement.as_mut().unwrap());
    }

    if !statements.contains_key(query) {
        let statement = SqliteStatement::prepare(conn, query, false)?;
        statements.insert(query.to_owned(), statement);
    }

    let statement = statements.get_mut(query).unwrap();

    // as this statement has been executed before, we reset before continuing
    // this also causes any rows that are from the statement to be inflated
    statement.reset();

    Ok(statement)
}

fn bind(
    statement: &mut SqliteStatement,
    arguments: Option<SqliteArguments<'_>>,
) -> Result<(), Error> {
    if let Some(arguments) = arguments {
        arguments.bind(&*statement)?;
    }

    Ok(())
}

fn emplace_row_metadata(
    statement: &StatementHandle,
    column_names: &mut HashMap<UStr, usize>,
) -> Result<(), Error> {
    column_names.clear();

    let num = statement.column_count();

    column_names.reserve(num);

    for i in 0..num {
        let name: UStr = statement.column_name(i).to_owned().into();

        column_names.insert(name, i);
    }

    Ok(())
}

impl<'c> Executor<'c> for &'c mut SqliteConnection {
    type Database = Sqlite;

    fn fetch_many<'e, 'q: 'e, E: 'q>(
        self,
        mut query: E,
    ) -> BoxStream<'e, Result<Either<u64, SqliteRow>, Error>>
    where
        'c: 'e,
        E: Execute<'q, Self::Database>,
    {
        let s = query.query();
        let arguments = query.take_arguments();

        Box::pin(try_stream! {
            let SqliteConnection {
                handle: ref mut conn,
                ref mut statements,
                ref mut statement,
                ref worker,
                ref mut scratch_row_column_names,
                ..
            } = self;

            // prepare statement object (or checkout from cache)
            let mut stmt = prepare(conn, statements, statement, s, arguments.is_some())?;

            // bind arguments, if any, to the statement
            bind(&mut stmt, arguments)?;

            while let Some((handle, last_row_values)) = stmt.execute()? {
                // tell the worker about the new statement
                worker.execute(handle);

                // wake up the worker if needed
                // the worker parks its thread on async-std when not in use
                worker.wake();

                emplace_row_metadata(
                    handle,
                    Arc::make_mut(scratch_row_column_names),
                )?;

                loop {
                    // save the rows from the _current_ position on the statement
                    // and send them to the still-live row object
                    SqliteRow::inflate_if_needed(handle, last_row_values.take());

                    match worker.step(handle).await? {
                        Either::Left(changes) => {
                            r#yield!(Either::Left(changes));

                            break;
                        }

                        Either::Right(()) => {
                            let (row, weak_values_ref) = SqliteRow::current(
                                *handle,
                                scratch_row_column_names
                            );

                            let v = Either::Right(row);
                            *last_row_values = Some(weak_values_ref);

                            r#yield!(v);
                        }
                    }
                }
            }

            Ok(())
        })
    }

    fn fetch_optional<'e, 'q: 'e, E: 'q>(
        self,
        query: E,
    ) -> BoxFuture<'e, Result<Option<SqliteRow>, Error>>
    where
        'c: 'e,
        E: Execute<'q, Self::Database>,
    {
        let mut s = self.fetch_many(query);

        Box::pin(async move {
            while let Some(v) = s.try_next().await? {
                if let Either::Right(r) = v {
                    return Ok(Some(r));
                }
            }

            Ok(None)
        })
    }

    #[doc(hidden)]
    fn describe<'e, 'q: 'e, E: 'q>(self, query: E) -> BoxFuture<'e, Result<Describe<Sqlite>, Error>>
    where
        'c: 'e,
        E: Execute<'q, Self::Database>,
    {
        let query = query.query();
        let statement = SqliteStatement::prepare(&mut self.handle, query, false);

        Box::pin(async move {
            let mut params = Vec::new();
            let mut columns = Vec::new();

            if let Some(statement) = statement?.handles.get(0) {
                // NOTE: we can infer *nothing* about parameters apart from the count
                params.resize(statement.bind_parameter_count(), None);

                let num_columns = statement.column_count();
                columns.reserve(num_columns);

                for i in 0..num_columns {
                    let name = statement.column_name(i).to_owned();
                    let type_info = statement.column_decltype(i);
                    let not_null = statement.column_not_null(i)?;

                    columns.push(Column {
                        name,
                        type_info,
                        not_null,
                    })
                }
            }

            Ok(Describe { params, columns })
        })
    }
}
