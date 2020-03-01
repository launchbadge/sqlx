use std::future::Future;
use std::mem;
use std::pin::Pin;
use std::sync::Arc;
use std::task::{Context, Poll};

use async_stream::try_stream;
use futures_core::future::BoxFuture;
use futures_core::stream::BoxStream;

use crate::connection::{ConnectionSource, MaybeOwnedConnection};
use crate::cursor::Cursor;
use crate::database::HasRow;
use crate::executor::Execute;
use crate::pool::{Pool, PoolConnection};
use crate::postgres::protocol::{CommandComplete, DataRow, Message, StatementId};
use crate::postgres::{PgArguments, PgConnection, PgRow};
use crate::{Database, Postgres};
use futures_core::Stream;

pub struct PgCursor<'c, 'q> {
    source: ConnectionSource<'c, PgConnection>,
    query: Option<(&'q str, Option<PgArguments>)>,
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
            query: Some(query.into_parts()),
        }
    }

    #[doc(hidden)]
    fn from_connection<E, C>(conn: C, query: E) -> Self
    where
        Self: Sized,
        C: Into<MaybeOwnedConnection<'c, PgConnection>>,
        E: Execute<'q, Postgres>,
    {
        Self {
            source: ConnectionSource::Connection(conn.into()),
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
    let mut conn = cursor.source.resolve_by_ref().await?;

    // The first time [next] is called we need to actually execute our
    // contained query. We guard against this happening on _all_ next calls
    // by using [Option::take] which replaces the potential value in the Option with `None
    if let Some((query, arguments)) = cursor.query.take() {
        conn.execute(query, arguments).await?;
    }

    loop {
        match conn.stream.read().await? {
            Message::ParseComplete | Message::BindComplete => {
                // ignore x_complete messages
            }

            Message::CommandComplete => {
                // no more rows
                break;
            }

            Message::DataRow => {
                let data = DataRow::read(&mut *conn)?;

                return Ok(Some(PgRow {
                    connection: conn,
                    columns: Arc::default(),
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
