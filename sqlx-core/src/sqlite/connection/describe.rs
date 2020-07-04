use crate::describe::{Column, Describe};
use crate::error::Error;
use crate::sqlite::connection::explain::explain;
use crate::sqlite::statement::SqliteStatement;
use crate::sqlite::type_info::DataType;
use crate::sqlite::{Sqlite, SqliteConnection, SqliteTypeInfo};
use futures_core::future::BoxFuture;

pub(super) async fn describe(
    conn: &mut SqliteConnection,
    query: &str,
) -> Result<Describe<Sqlite>, Error> {
    describe_with(conn, query, vec![]).await
}

pub(super) fn describe_with<'c: 'e, 'q: 'e, 'e>(
    conn: &'c mut SqliteConnection,
    query: &'q str,
    fallback: Vec<SqliteTypeInfo>,
) -> BoxFuture<'e, Result<Describe<Sqlite>, Error>> {
    Box::pin(async move {
        // describing a statement from SQLite can be involved
        // each SQLx statement is comprised of multiple SQL statements

        let SqliteConnection {
            ref mut handle,
            ref worker,
            ..
        } = conn;

        let statement = SqliteStatement::prepare(handle, query, false);

        let mut columns = Vec::new();
        let mut num_params = 0;

        let mut statement = statement?;

        // we start by finding the first statement that *can* return results
        while let Some((statement, _)) = statement.execute()? {
            num_params += statement.bind_parameter_count();

            let mut stepped = false;

            let num = statement.column_count();
            if num == 0 {
                // no columns in this statement; skip
                continue;
            }

            // next we try to use [column_decltype] to inspect the type of each column
            columns.reserve(num);

            for col in 0..num {
                let name = statement.column_name(col).to_owned();

                let type_info = if let Some(ty) = statement.column_decltype(col) {
                    ty
                } else {
                    // if that fails, we back up and attempt to step the statement
                    // once *if* its read-only and then use [column_type] as a
                    // fallback to [column_decltype]
                    if !stepped && statement.read_only() && fallback.is_empty() {
                        stepped = true;

                        worker.execute(statement);
                        worker.wake();

                        let _ = worker.step(statement).await?;
                    }

                    let mut ty = statement.column_type_info(col);

                    if ty.0 == DataType::Null {
                        if fallback.is_empty() {
                            // this will _still_ fail if there are no actual rows to return
                            // this happens more often than not for the macros as we tell
                            // users to execute against an empty database

                            // as a last resort, we explain the original query and attempt to
                            // infer what would the expression types be as a fallback
                            // to [column_decltype]

                            let fallback = explain(conn, statement.sql()).await?;

                            return describe_with(conn, query, fallback).await;
                        }

                        if let Some(fallback) = fallback.get(col).cloned() {
                            ty = fallback;
                        }
                    }

                    ty
                };

                let not_null = statement.column_not_null(col)?;

                columns.push(Column {
                    name,
                    type_info: Some(type_info),
                    not_null,
                });
            }
        }

        // println!("describe ->> {:#?}", columns);

        Ok(Describe {
            columns,
            params: vec![None; num_params],
        })
    })
}
