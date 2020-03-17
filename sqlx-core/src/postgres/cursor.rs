use std::collections::HashMap;
use std::sync::Arc;

use futures_core::future::BoxFuture;

use crate::connection::ConnectionSource;
use crate::cursor::Cursor;
use crate::executor::Execute;
use crate::pool::Pool;
use crate::postgres::protocol::{
    DataRow, Message, ReadyForQuery, RowDescription, StatementId, TypeFormat,
};
use crate::postgres::{PgArguments, PgConnection, PgRow, Postgres};

pub struct PgCursor<'c, 'q> {
    source: ConnectionSource<'c, PgConnection>,
    query: Option<(&'q str, Option<PgArguments>)>,
    columns: Arc<HashMap<Box<str>, usize>>,
    formats: Arc<[TypeFormat]>,
}

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
            columns: Arc::default(),
            formats: Arc::new([] as [TypeFormat; 0]),
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
            columns: Arc::default(),
            formats: Arc::new([] as [TypeFormat; 0]),
            query: Some(query.into_parts()),
        }
    }

    fn next(&mut self) -> BoxFuture<crate::Result<Option<PgRow<'_>>>> {
        Box::pin(next(self))
    }
}

fn parse_row_description(rd: RowDescription) -> (HashMap<Box<str>, usize>, Vec<TypeFormat>) {
    let mut columns = HashMap::new();
    let mut formats = Vec::new();

    columns.reserve(rd.fields.len());
    formats.reserve(rd.fields.len());

    for (index, field) in rd.fields.iter().enumerate() {
        if let Some(name) = &field.name {
            columns.insert(name.clone(), index);
        }

        formats.push(field.type_format);
    }

    (columns, formats)
}

// Used to describe the incoming results
// We store the column map in an Arc and share it among all rows
async fn expect_desc(
    conn: &mut PgConnection,
) -> crate::Result<(HashMap<Box<str>, usize>, Vec<TypeFormat>)> {
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
    statement: StatementId,
) -> crate::Result<(Arc<HashMap<Box<str>, usize>>, Arc<[TypeFormat]>)> {
    if !conn.cache_statement_columns.contains_key(&statement)
        || !conn.cache_statement_formats.contains_key(&statement)
    {
        let (columns, formats) = expect_desc(conn).await?;

        conn.cache_statement_columns
            .insert(statement, Arc::new(columns));

        conn.cache_statement_formats
            .insert(statement, Arc::from(formats));
    }

    Ok((
        Arc::clone(&conn.cache_statement_columns[&statement]),
        Arc::clone(&conn.cache_statement_formats[&statement]),
    ))
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
            let (columns, formats) = get_or_describe(&mut *conn, statement).await?;

            cursor.columns = columns;
            cursor.formats = formats;
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
                let (columns, formats) = parse_row_description(rd);

                cursor.columns = Arc::new(columns);
                cursor.formats = Arc::from(formats);
            }

            Message::DataRow => {
                let data = DataRow::read(conn.stream.buffer(), &mut conn.current_row_values)?;

                return Ok(Some(PgRow {
                    columns: Arc::clone(&cursor.columns),
                    formats: Arc::clone(&cursor.formats),
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
