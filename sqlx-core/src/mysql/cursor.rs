use std::collections::HashMap;
use std::sync::Arc;

use futures_core::future::BoxFuture;

use crate::connection::{ConnectionSource, MaybeOwnedConnection};
use crate::cursor::Cursor;
use crate::executor::Execute;
use crate::mysql::protocol::{ColumnCount, ColumnDefinition, Decode, Row, Status, TypeId};
use crate::mysql::{MySql, MySqlArguments, MySqlConnection, MySqlRow};
use crate::pool::Pool;

pub struct MySqlCursor<'c, 'q> {
    source: ConnectionSource<'c, MySqlConnection>,
    query: Option<(&'q str, Option<MySqlArguments>)>,
    column_names: Arc<HashMap<Box<str>, u16>>,
    column_types: Vec<TypeId>,
    binary: bool,
}

impl<'c, 'q> Cursor<'c, 'q> for MySqlCursor<'c, 'q> {
    type Database = MySql;

    #[doc(hidden)]
    fn from_pool<E>(pool: &Pool<MySqlConnection>, query: E) -> Self
    where
        Self: Sized,
        E: Execute<'q, MySql>,
    {
        Self {
            source: ConnectionSource::Pool(pool.clone()),
            column_names: Arc::default(),
            column_types: Vec::new(),
            binary: true,
            query: Some(query.into_parts()),
        }
    }

    #[doc(hidden)]
    fn from_connection<E, C>(conn: C, query: E) -> Self
    where
        Self: Sized,
        C: Into<MaybeOwnedConnection<'c, MySqlConnection>>,
        E: Execute<'q, MySql>,
    {
        Self {
            source: ConnectionSource::Connection(conn.into()),
            column_names: Arc::default(),
            column_types: Vec::new(),
            binary: true,
            query: Some(query.into_parts()),
        }
    }

    fn next(&mut self) -> BoxFuture<crate::Result<Option<MySqlRow<'_>>>> {
        Box::pin(next(self))
    }
}

async fn next<'a, 'c: 'a, 'q: 'a>(
    cursor: &'a mut MySqlCursor<'c, 'q>,
) -> crate::Result<Option<MySqlRow<'a>>> {
    println!("[cursor::next]");

    let mut conn = cursor.source.resolve_by_ref().await?;

    // The first time [next] is called we need to actually execute our
    // contained query. We guard against this happening on _all_ next calls
    // by using [Option::take] which replaces the potential value in the Option with `None
    let mut initial = if let Some((query, arguments)) = cursor.query.take() {
        let statement = conn.run(query, arguments).await?;

        // No statement ID = TEXT mode
        cursor.binary = statement.is_some();

        true
    } else {
        false
    };

    loop {
        let mut packet_id = conn.stream.receive().await?[0];
        println!("[cursor::next/iter] {:x}", packet_id);
        match packet_id {
            // OK or EOF packet
            0x00 | 0xFE
                if conn.stream.packet().len() < 0xFF_FF_FF && (packet_id != 0x00 || initial) =>
            {
                let ok = conn.stream.handle_ok()?;

                if ok.status.contains(Status::SERVER_MORE_RESULTS_EXISTS) {
                    // There is more to this query
                    initial = true;
                } else {
                    conn.is_ready = true;
                    return Ok(None);
                }
            }

            // ERR packet
            0xFF => {
                conn.is_ready = true;
                return conn.stream.handle_err();
            }

            _ if initial => {
                // At the start of the results we expect to see a
                // COLUMN_COUNT followed by N COLUMN_DEF

                let cc = ColumnCount::decode(conn.stream.packet())?;

                // We use these definitions to get the actual column types that is critical
                // in parsing the rows coming back soon

                cursor.column_types.clear();
                cursor.column_types.reserve(cc.columns as usize);

                let mut column_names = HashMap::with_capacity(cc.columns as usize);

                for i in 0..cc.columns {
                    let column = ColumnDefinition::decode(conn.stream.receive().await?)?;

                    cursor.column_types.push(column.type_id);

                    if let Some(name) = column.name() {
                        column_names.insert(name.to_owned().into_boxed_str(), i as u16);
                    }
                }

                cursor.column_names = Arc::new(column_names);
                initial = false;
            }

            _ if !cursor.binary || packet_id == 0x00 => {
                let row = Row::read(
                    conn.stream.packet(),
                    &cursor.column_types,
                    &mut conn.current_row_values,
                    // TODO: Text mode
                    cursor.binary,
                )?;

                let row = MySqlRow {
                    row,
                    columns: Arc::clone(&cursor.column_names),
                    // TODO: Text mode
                    binary: cursor.binary,
                };

                return Ok(Some(row));
            }

            _ => {
                return conn.stream.handle_unexpected();
            }
        }
    }
}
