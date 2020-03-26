use std::collections::HashMap;
use std::sync::Arc;

use futures_core::future::BoxFuture;

use crate::connection::ConnectionSource;
use crate::cursor::Cursor;
use crate::executor::Execute;
use crate::pool::Pool;
use crate::postgres::protocol::{DataRow, Message, ReadyForQuery, RowDescription, StatementId};
use crate::postgres::row::{Column, Statement};
use crate::postgres::{PgArguments, PgConnection, PgRow, Postgres};

pub struct PgCursor<'c, 'q> {
    source: ConnectionSource<'c, PgConnection>,
    query: Option<(&'q str, Option<PgArguments>)>,
    statement: Arc<Statement>,
}

impl crate::cursor::private::Sealed for PgCursor<'_, '_> {}

impl<'c, 'q> Cursor<'c, 'q> for PgCursor<'c, 'q> {
    type Database = Postgres;

    #[doc(hidden)]
    fn from_pool<E>(pool: &Pool<PgConnection>, query: E) -> Self
    where
        Self: Sized,
        E: Execute<'q, Postgres>,
    {
        Self {
            source: ConnectionSource::Pool(pool.clone()),
            statement: Arc::default(),
            query: Some(query.into_parts()),
        }
    }

    #[doc(hidden)]
    fn from_connection<E>(conn: &'c mut PgConnection, query: E) -> Self
    where
        Self: Sized,
        E: Execute<'q, Postgres>,
    {
        Self {
            source: ConnectionSource::ConnectionRef(conn),
            statement: Arc::default(),
            query: Some(query.into_parts()),
        }
    }

    fn next(&mut self) -> BoxFuture<crate::Result<Option<PgRow<'_>>>> {
        Box::pin(next(self))
    }
}

fn parse_row_description(rd: RowDescription) -> Statement {
    let mut names = HashMap::new();
    let mut columns = Vec::new();

    columns.reserve(rd.fields.len());
    names.reserve(rd.fields.len());

    for (index, field) in rd.fields.iter().enumerate() {
        if let Some(name) = &field.name {
            names.insert(name.clone(), index);
        }

        columns.push(Column {
            type_id: field.type_id,
            format: field.type_format,
        });
    }

    Statement {
        columns: columns.into_boxed_slice(),
        names,
    }
}

// Used to describe the incoming results
// We store the column map in an Arc and share it among all rows
async fn expect_desc(conn: &mut PgConnection) -> crate::Result<Statement> {
    let description: Option<_> = loop {
        match conn.stream.receive().await? {
            Message::ParseComplete | Message::BindComplete => {}

            Message::RowDescription => {
                break Some(RowDescription::read(conn.stream.buffer())?);
            }

            Message::NoData => {
                break None;
            }

            message => {
                return Err(
                    protocol_err!("next/describe: unexpected message: {:?}", message).into(),
                );
            }
        }
    };

    Ok(description.map(parse_row_description).unwrap_or_default())
}

// A form of describe that uses the statement cache
async fn get_or_describe(
    conn: &mut PgConnection,
    id: StatementId,
) -> crate::Result<Arc<Statement>> {
    if !conn.cache_statement.contains_key(&id) {
        let statement = expect_desc(conn).await?;

        conn.cache_statement.insert(id, Arc::new(statement));
    }

    Ok(Arc::clone(&conn.cache_statement[&id]))
}

async fn next<'a, 'c: 'a, 'q: 'a>(
    cursor: &'a mut PgCursor<'c, 'q>,
) -> crate::Result<Option<PgRow<'a>>> {
    let mut conn = cursor.source.resolve().await?;

    // The first time [next] is called we need to actually execute our
    // contained query. We guard against this happening on _all_ next calls
    // by using [Option::take] which replaces the potential value in the Option with `None
    if let Some((query, arguments)) = cursor.query.take() {
        let statement = conn.run(query, arguments).await?;

        // If there is a statement ID, this is a non-simple or prepared query
        if let Some(statement) = statement {
            // A prepared statement will re-use the previous column map if
            // this query has been executed before
            cursor.statement = get_or_describe(&mut *conn, statement).await?;
        }

        // A non-prepared query must be described each time
        // We wait until we hit a RowDescription
    }

    loop {
        match conn.stream.receive().await? {
            // Indicates that a phase of the extended query flow has completed
            // We as SQLx don't generally care as long as it is happening
            Message::ParseComplete | Message::BindComplete => {}

            // Indicates that _a_ query has finished executing
            Message::CommandComplete => {}

            // Indicates that all queries have finished executing
            Message::ReadyForQuery => {
                // TODO: How should we handle an ERROR status form ReadyForQuery
                let _ready = ReadyForQuery::read(conn.stream.buffer())?;

                conn.is_ready = true;
                break;
            }

            Message::RowDescription => {
                let rd = RowDescription::read(conn.stream.buffer())?;
                cursor.statement = Arc::new(parse_row_description(rd));
            }

            Message::DataRow => {
                let data = DataRow::read(conn.stream.buffer(), &mut conn.current_row_values)?;

                return Ok(Some(PgRow {
                    statement: Arc::clone(&cursor.statement),
                    data,
                }));
            }

            message => {
                return Err(protocol_err!("next: unexpected message: {:?}", message).into());
            }
        }
    }

    Ok(None)
}
