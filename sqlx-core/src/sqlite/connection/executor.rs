use std::sync::Arc;

use either::Either;
use futures_core::future::BoxFuture;
use futures_core::stream::BoxStream;
use futures_util::TryStreamExt;
use hashbrown::HashMap;
use libsqlite3_sys::sqlite3_last_insert_rowid;

use crate::common::StatementCache;
use crate::error::Error;
use crate::executor::{Execute, Executor};
use crate::ext::ustr::UStr;
use crate::sqlite::connection::describe::describe;
use crate::sqlite::connection::ConnectionHandle;
use crate::sqlite::statement::{SqliteStatement, StatementHandle};
use crate::sqlite::{
    type_info::DataType, Sqlite, SqliteArguments, SqliteColumn, SqliteConnection, SqliteDone,
    SqliteRow, SqliteTypeInfo,
};
use crate::statement::StatementInfo;

fn prepare<'a>(
    conn: &mut ConnectionHandle,
    statements: &'a mut StatementCache<SqliteStatement>,
    statement: &'a mut Option<SqliteStatement>,
    query: &str,
    persistent: bool,
) -> Result<&'a mut SqliteStatement, Error> {
    if !persistent || statements.capacity() == 0 {
        *statement = Some(SqliteStatement::prepare(conn, query, false)?);
        return Ok(statement.as_mut().unwrap());
    }

    let exists = statements.contains_key(query);

    if !exists {
        let statement = SqliteStatement::prepare(conn, query, true)?;
        statements.insert(query, statement);
    }

    let statement = statements.get_mut(query).unwrap();

    if exists {
        // as this statement has been executed before, we reset before continuing
        // this also causes any rows that are from the statement to be inflated
        statement.reset();
    }

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
    columns: &mut Vec<SqliteColumn>,
    column_names: &mut HashMap<UStr, usize>,
) -> Result<(), Error> {
    columns.clear();
    column_names.clear();

    let num = statement.column_count();

    column_names.reserve(num);
    columns.reserve(num);

    for i in 0..num {
        let name: UStr = statement.column_name(i).to_owned().into();
        let type_info = statement.column_type_info(i);

        columns.push(SqliteColumn {
            ordinal: i,
            name: name.clone(),
            type_info,
        });

        column_names.insert(name, i);
    }

    Ok(())
}

fn update_column_type_metadata(statement: &StatementHandle, columns: &mut Vec<SqliteColumn>) {
    for col in columns.iter_mut() {
        col.type_info = statement.column_type_info(col.ordinal);
    }
}

impl<'c> Executor<'c> for &'c mut SqliteConnection {
    type Database = Sqlite;

    fn fetch_many<'e, 'q: 'e, E: 'q>(
        self,
        mut query: E,
    ) -> BoxStream<'e, Result<Either<SqliteDone, SqliteRow>, Error>>
    where
        'c: 'e,
        E: Execute<'q, Self::Database>,
    {
        let s = query.query();
        let arguments = query.take_arguments();
        let persistent = query.persistent() && arguments.is_some();

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
            let mut stmt = prepare(conn, statements, statement, s, persistent)?;

            // bind arguments, if any, to the statement
            bind(&mut stmt, arguments)?;

            while let Some((handle, columns, last_row_values)) = stmt.execute()? {
                let mut have_metadata = false;

                // tell the worker about the new statement
                worker.execute(handle);

                // wake up the worker if needed
                // the worker parks its thread on async-std when not in use
                worker.wake();

                loop {
                    // save the rows from the _current_ position on the statement
                    // and send them to the still-live row object
                    SqliteRow::inflate_if_needed(handle, &*columns, last_row_values.take());

                    let s = worker.step(handle).await?;

                    if !have_metadata {
                        have_metadata = true;

                        emplace_row_metadata(
                            handle,
                            Arc::make_mut(columns),
                            Arc::make_mut(scratch_row_column_names),
                        )?;
                    } else {
                        update_column_type_metadata(handle, Arc::make_mut(columns));
                    }

                    match s {
                        Either::Left(changes) => {
                            let last_insert_rowid = unsafe {
                                sqlite3_last_insert_rowid(conn.as_ptr())
                            };

                            let done = SqliteDone {
                                changes: changes,
                                last_insert_rowid: last_insert_rowid,
                            };

                            r#yield!(Either::Left(done));

                            break;
                        }

                        Either::Right(()) => {
                            let (row, weak_values_ref) = SqliteRow::current(
                                *handle,
                                columns,
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

    fn describe<'e, 'q: 'e, E: 'q>(
        self,
        query: E,
    ) -> BoxFuture<'e, Result<StatementInfo<Sqlite>, Error>>
    where
        'c: 'e,
        E: Execute<'q, Self::Database>,
    {
        Box::pin(async move {
            let persistent = query.persistent();
            let query = query.query();

            let SqliteConnection {
                handle: ref mut conn,
                ref mut statements,
                ref mut statement,
                ..
            } = self;

            // prepare statement object (or checkout from cache)
            let statement = prepare(conn, statements, statement, query, persistent)?;

            let mut params = 0;
            let mut columns = Vec::new();

            if let Some(statement) = statement.handles.get(0) {
                // NOTE: we can infer *nothing* about parameters apart from the count
                params = statement.bind_parameter_count();

                let num_columns = statement.column_count();
                columns.reserve(num_columns);

                for i in 0..num_columns {
                    let name = statement.column_name(i).to_owned().into();

                    let type_info = statement
                        .column_decltype(i)
                        .unwrap_or(SqliteTypeInfo(DataType::Null));

                    let ordinal = i;

                    columns.push(SqliteColumn {
                        name,
                        type_info,
                        ordinal,
                    })
                }
            }

            let parameters = Some(Either::Right(params));
            let nullable = Vec::new();

            Ok(StatementInfo {
                parameters,
                columns,
                nullable,
            })
        })
    }

    #[doc(hidden)]
    fn describe_full<'e, 'q: 'e, E: 'q>(
        self,
        query: E,
    ) -> BoxFuture<'e, Result<StatementInfo<Sqlite>, Error>>
    where
        'c: 'e,
        E: Execute<'q, Self::Database>,
    {
        describe(self, query.query())
    }
}
