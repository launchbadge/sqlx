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

enum State<'c, 'q> {
    Query(&'q str, Option<PgArguments>),
    NextRow,

    // Used for `impl Future`
    Resolve(BoxFuture<'c, crate::Result<MaybeOwnedConnection<'c, PgConnection>>>),
    AffectedRows(BoxFuture<'c, crate::Result<u64>>),
}

pub struct PgCursor<'c, 'q> {
    source: ConnectionSource<'c, PgConnection>,
    state: State<'c, 'q>,
}

impl<'c, 'q> Cursor<'c, 'q, Postgres> for PgCursor<'c, 'q> {
    #[doc(hidden)]
    fn from_pool<E>(pool: &Pool<PgConnection>, query: E) -> Self
    where
        Self: Sized,
        E: Execute<'q, Postgres>,
    {
        let (query, arguments) = query.into_parts();

        Self {
            // note: pool is internally reference counted
            source: ConnectionSource::Pool(pool.clone()),
            state: State::Query(query, arguments),
        }
    }

    #[doc(hidden)]
    fn from_connection<E, C>(conn: C, query: E) -> Self
    where
        Self: Sized,
        C: Into<MaybeOwnedConnection<'c, PgConnection>>,
        E: Execute<'q, Postgres>,
    {
        let (query, arguments) = query.into_parts();

        Self {
            // note: pool is internally reference counted
            source: ConnectionSource::Connection(conn.into()),
            state: State::Query(query, arguments),
        }
    }

    #[doc(hidden)]
    fn first(self) -> BoxFuture<'c, crate::Result<Option<PgRow<'c>>>>
    where
        'q: 'c,
    {
        Box::pin(first(self))
    }

    fn next(&mut self) -> BoxFuture<crate::Result<Option<PgRow<'_>>>> {
        Box::pin(next(self))
    }
}

impl<'s, 'q> Future for PgCursor<'s, 'q> {
    type Output = crate::Result<u64>;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        loop {
            match &mut self.state {
                State::Query(q, arguments) => {
                    // todo: existential types can remove both the boxed futures
                    //       and this allocation
                    let query = q.to_owned();
                    let arguments = mem::take(arguments);

                    self.state = State::Resolve(Box::pin(resolve(
                        mem::take(&mut self.source),
                        query,
                        arguments,
                    )));
                }

                State::Resolve(fut) => {
                    match fut.as_mut().poll(cx) {
                        Poll::Pending => {
                            return Poll::Pending;
                        }

                        Poll::Ready(conn) => {
                            let conn = conn?;

                            self.state = State::AffectedRows(Box::pin(affected_rows(conn)));

                            // continue
                        }
                    }
                }

                State::NextRow => {
                    panic!("PgCursor must not be polled after being used");
                }

                State::AffectedRows(fut) => {
                    return fut.as_mut().poll(cx);
                }
            }
        }
    }
}

// write out query to the connection stream
async fn write(
    conn: &mut PgConnection,
    query: &str,
    arguments: Option<PgArguments>,
) -> crate::Result<()> {
    if let Some(arguments) = arguments {
        // Check the statement cache for a statement ID that matches the given query
        // If it doesn't exist, we generate a new statement ID and write out [Parse] to the
        // connection command buffer
        let statement = conn.write_prepare(query, &arguments);

        // Next, [Bind] attaches the arguments to the statement and creates a named portal
        conn.write_bind("", statement, &arguments);

        // Next, [Describe] will return the expected result columns and types
        // Conditionally run [Describe] only if the results have not been cached
        // if !self.statement_cache.has_columns(statement) {
        //     self.write_describe(protocol::Describe::Portal(""));
        // }

        // Next, [Execute] then executes the named portal
        conn.write_execute("", 0);

        // Finally, [Sync] asks postgres to process the messages that we sent and respond with
        // a [ReadyForQuery] message when it's completely done. Theoretically, we could send
        // dozens of queries before a [Sync] and postgres can handle that. Execution on the server
        // is still serial but it would reduce round-trips. Some kind of builder pattern that is
        // termed batching might suit this.
        conn.write_sync();
    } else {
        // https://www.postgresql.org/docs/12/protocol-flow.html#id-1.10.5.7.4
        conn.write_simple_query(query);
    }

    conn.wait_until_ready().await?;

    conn.stream.flush().await?;
    conn.is_ready = false;

    Ok(())
}

async fn resolve(
    mut source: ConnectionSource<'_, PgConnection>,
    query: String,
    arguments: Option<PgArguments>,
) -> crate::Result<MaybeOwnedConnection<'_, PgConnection>> {
    let mut conn = source.resolve_by_ref().await?;

    write(&mut *conn, &query, arguments).await?;

    Ok(source.into_connection())
}

async fn affected_rows(mut conn: MaybeOwnedConnection<'_, PgConnection>) -> crate::Result<u64> {
    let mut rows = 0;

    loop {
        match conn.stream.read().await? {
            Message::ParseComplete | Message::BindComplete => {
                // ignore x_complete messages
            }

            Message::DataRow => {
                // ignore rows
                // TODO: should we log or something?
            }

            Message::CommandComplete => {
                rows += CommandComplete::read(conn.stream.buffer())?.affected_rows;
            }

            Message::ReadyForQuery => {
                // done
                conn.is_ready = true;
                break;
            }

            message => {
                return Err(
                    protocol_err!("affected_rows: unexpected message: {:?}", message).into(),
                );
            }
        }
    }

    Ok(rows)
}

async fn next<'a, 'c: 'a, 'q: 'a>(
    cursor: &'a mut PgCursor<'c, 'q>,
) -> crate::Result<Option<PgRow<'a>>> {
    let mut conn = cursor.source.resolve_by_ref().await?;

    match cursor.state {
        State::Query(q, ref mut arguments) => {
            // write out the query to the connection
            write(&mut *conn, q, arguments.take()).await?;

            // next time we come through here, skip this block
            cursor.state = State::NextRow;
        }

        State::Resolve(_) | State::AffectedRows(_) => {
            panic!("`PgCursor` must not be used after being polled");
        }

        State::NextRow => {
            // grab the next row
        }
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

async fn first<'c, 'q>(mut cursor: PgCursor<'c, 'q>) -> crate::Result<Option<PgRow<'c>>> {
    let mut conn = cursor.source.resolve().await?;

    match cursor.state {
        State::Query(q, ref mut arguments) => {
            // write out the query to the connection
            write(&mut conn, q, arguments.take()).await?;
        }

        State::NextRow => {
            // just grab the next row as the first
        }

        State::Resolve(_) | State::AffectedRows(_) => {
            panic!("`PgCursor` must not be used after being polled");
        }
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
                let data = DataRow::read(&mut conn)?;

                return Ok(Some(PgRow {
                    connection: conn,
                    columns: Arc::default(),
                    data,
                }));
            }

            message => {
                return Err(protocol_err!("first: unexpected message: {:?}", message).into());
            }
        }
    }

    Ok(None)
}
