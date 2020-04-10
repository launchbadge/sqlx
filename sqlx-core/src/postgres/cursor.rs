use std::sync::Arc;

use futures_core::future::BoxFuture;

use crate::connection::ConnectionSource;
use crate::cursor::Cursor;
use crate::executor::Execute;
use crate::pool::Pool;
use crate::postgres::protocol::{DataRow, Message, ReadyForQuery, RowDescription};
use crate::postgres::row::Statement;
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
            // A prepared statement will re-use the previous column map
            cursor.statement = Arc::clone(&conn.cache_statement[&statement]);
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
                // NOTE: This is only encountered for unprepared statements
                let rd = RowDescription::read(conn.stream.buffer())?;
                cursor.statement = Arc::new(
                    conn.parse_row_description(rd, Default::default(), None, false)
                        .await?,
                );
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
