use super::{
    protocol::{self, BufMut, Message},
    Pg, PgConnection, PgRow,
};
use crate::{
    query::Query,
    row::FromRow,
    serialize::{IsNull, ToSql},
    types::HasSqlType,
};
use byteorder::{BigEndian, ByteOrder};
use futures::{
    future::BoxFuture,
    stream::{self, BoxStream},
    Stream,
};
use std::io;

pub struct PgQuery<'c, 'q> {
    conn: &'c mut PgConnection,
    query: &'q str,
    // OIDs of the bind parameters
    types: Vec<u32>,
    // Write buffer for serializing bind values
    buf: Vec<u8>,
}

impl<'c, 'q> PgQuery<'c, 'q> {
    pub(super) fn new(conn: &'c mut PgConnection, query: &'q str) -> Self {
        Self {
            query,
            conn,
            types: Vec::new(),
            buf: Vec::new(),
        }
    }

    fn finish(self, limit: i32) -> &'c mut PgConnection {
        self.conn.write(protocol::Parse {
            portal: "",
            query: self.query,
            param_types: &*self.types,
        });

        self.conn.write(protocol::Bind {
            portal: "",
            statement: "",
            formats: &[1], // [BINARY]
            // TODO: Early error if there is more than i16
            values_len: self.types.len() as i16,
            values: &*self.buf,
            result_formats: &[1], // [BINARY]
        });

        self.conn.write(protocol::Execute { portal: "", limit });

        self.conn.write(protocol::Sync);

        self.conn
    }
}

impl<'c, 'q> Query<'c, 'q> for PgQuery<'c, 'q> {
    type Backend = Pg;

    fn bind_as<ST, T>(mut self, value: T) -> Self
    where
        Self: Sized,
        Self::Backend: HasSqlType<ST>,
        T: ToSql<ST, Self::Backend>,
    {
        // TODO: When/if we receive types that do _not_ support BINARY, we need to check here
        // TODO: There is no need to be explicit unless we are expecting mixed BINARY / TEXT

        self.types.push(<Pg as HasSqlType<ST>>::metadata().oid);

        let pos = self.buf.len();
        self.buf.put_int_32(0);

        let len = if let IsNull::No = value.to_sql(&mut self.buf) {
            (self.buf.len() - pos - 4) as i32
        } else {
            // Write a -1 for the len to indicate NULL
            // TODO: It is illegal for [to_sql] to write any data if IsSql::No; fail a debug assertion
            -1
        };

        // Write-back the len to the beginning of this frame (not including the len of len)
        BigEndian::write_i32(&mut self.buf[pos..], len as i32);

        self
    }

    #[inline]
    fn execute(self) -> BoxFuture<'c, io::Result<u64>> {
        Box::pin(execute(self.finish(0)))
    }

    #[inline]
    fn fetch<A: 'c, T: 'c>(self) -> BoxStream<'c, io::Result<T>>
    where
        T: FromRow<A, Self::Backend>,
    {
        Box::pin(fetch(self.finish(0)))
    }

    #[inline]
    fn fetch_optional<A: 'c, T: 'c>(self) -> BoxFuture<'c, io::Result<Option<T>>>
    where
        T: FromRow<A, Self::Backend>,
    {
        Box::pin(fetch_optional(self.finish(1)))
    }
}

async fn execute(conn: &mut PgConnection) -> io::Result<u64> {
    conn.flush().await?;

    let mut rows = 0;

    while let Some(message) = conn.receive().await? {
        match message {
            Message::BindComplete | Message::ParseComplete | Message::DataRow(_) => {}

            Message::CommandComplete(body) => {
                rows = body.rows();
            }

            Message::ReadyForQuery(_) => {
                // Successful completion of the whole cycle
                return Ok(rows);
            }

            message => {
                unimplemented!("received {:?} unimplemented message", message);
            }
        }
    }

    // FIXME: This is an end-of-file error. How we should bubble this up here?
    unreachable!()
}

async fn fetch_optional<'a, A: 'a, T: 'a>(conn: &'a mut PgConnection) -> io::Result<Option<T>>
where
    T: FromRow<A, Pg>,
{
    conn.flush().await?;

    let mut row: Option<PgRow> = None;

    while let Some(message) = conn.receive().await? {
        match message {
            Message::BindComplete
            | Message::ParseComplete
            | Message::PortalSuspended
            | Message::CloseComplete
            | Message::CommandComplete(_) => {}

            Message::DataRow(body) => {
                row = Some(PgRow(body));
            }

            Message::ReadyForQuery(_) => {
                return Ok(row.map(T::from_row));
            }

            message => {
                unimplemented!("received {:?} unimplemented message", message);
            }
        }
    }

    // FIXME: This is an end-of-file error. How we should bubble this up here?
    unreachable!()
}

fn fetch<'a, A: 'a, T: 'a>(
    conn: &'a mut PgConnection,
) -> impl Stream<Item = Result<T, io::Error>> + 'a + Unpin
where
    T: FromRow<A, Pg>,
{
    // FIXME: Manually implement Stream on a new type to avoid the unfold adapter
    stream::unfold(conn, |conn| {
        Box::pin(async {
            if !conn.wbuf.is_empty() {
                if let Err(e) = conn.flush().await {
                    return Some((Err(e), conn));
                }
            }

            loop {
                let message = match conn.receive().await {
                    Ok(Some(message)) => message,
                    // FIXME: This is an end-of-file error. How we should bubble this up here?
                    Ok(None) => unreachable!(),
                    Err(e) => return Some((Err(e), conn)),
                };

                match message {
                    Message::BindComplete
                    | Message::ParseComplete
                    | Message::CommandComplete(_) => {}

                    Message::DataRow(row) => {
                        break Some((Ok(T::from_row(PgRow(row))), conn));
                    }

                    Message::ReadyForQuery(_) => {
                        // Successful completion of the whole cycle
                        break None;
                    }

                    message => {
                        unimplemented!("received {:?} unimplemented message", message);
                    }
                }
            }
        })
    })
}
