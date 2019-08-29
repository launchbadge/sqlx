use super::{
    protocol::{self, Decode, Encode, Message, Terminate},
    Postgres, PostgresQueryParameters, PostgresRow,
};
use crate::{connection::RawConnection, error::Error, io::BufStream, query::QueryParameters};
// use bytes::{BufMut, BytesMut};
use crate::{io::Buf, url::Url};
use byteorder::NetworkEndian;
use futures_core::{future::BoxFuture, stream::BoxStream};
use std::{
    io,
    net::{IpAddr, Shutdown, SocketAddr},
};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::TcpStream,
};

mod establish;
mod execute;
mod fetch;
mod fetch_optional;

pub struct PostgresRawConnection {
    stream: BufStream<TcpStream>,

    // Process ID of the Backend
    process_id: u32,

    // Backend-unique key to use to send a cancel query message to the server
    secret_key: u32,
}

impl PostgresRawConnection {
    async fn establish(url: &str) -> Result<Self, Error> {
        let url = Url::parse(url);
        let stream = TcpStream::connect(&url.address(5432)).await?;

        let mut conn = Self {
            stream: BufStream::new(stream),
            process_id: 0,
            secret_key: 0,
        };

        establish::establish(&mut conn, &url).await?;

        Ok(conn)
    }

    async fn finalize(&mut self) -> Result<(), Error> {
        self.write(Terminate);
        self.stream.flush().await?;
        self.stream.close().await?;

        Ok(())
    }

    // Wait and return the next message to be received from Postgres.
    async fn receive(&mut self) -> Result<Option<Message>, Error> {
        loop {
            // Read the message header (id + len)
            let mut header = ret_if_none!(self.stream.peek(5).await?);
            log::trace!("recv:header {:?}", bytes::Bytes::from(&*header));

            let id = header.get_u8()?;
            let len = (header.get_u32::<NetworkEndian>()? - 4) as usize;

            // Read the message body
            self.stream.consume(5);
            let body = ret_if_none!(self.stream.peek(len).await?);
            log::trace!("recv {:?}", bytes::Bytes::from(&*body));

            let message = match id {
                b'N' | b'E' => Message::Response(Box::new(protocol::Response::decode(body)?)),
                b'D' => Message::DataRow(protocol::DataRow::decode(body)?),
                b'S' => {
                    Message::ParameterStatus(Box::new(protocol::ParameterStatus::decode(body)?))
                }
                b'Z' => Message::ReadyForQuery(protocol::ReadyForQuery::decode(body)?),
                b'R' => Message::Authentication(Box::new(protocol::Authentication::decode(body)?)),
                b'K' => Message::BackendKeyData(protocol::BackendKeyData::decode(body)?),
                b'C' => Message::CommandComplete(protocol::CommandComplete::decode(body)?),
                b'A' => Message::NotificationResponse(Box::new(
                    protocol::NotificationResponse::decode(body)?,
                )),
                b'1' => Message::ParseComplete,
                b'2' => Message::BindComplete,
                b'3' => Message::CloseComplete,
                b'n' => Message::NoData,
                b's' => Message::PortalSuspended,
                b't' => Message::ParameterDescription(Box::new(
                    protocol::ParameterDescription::decode(body)?,
                )),

                _ => unimplemented!("unknown message id: {}", id as char),
            };

            self.stream.consume(len);

            match message {
                Message::ParameterStatus(_body) => {
                    // TODO: not sure what to do with these yet
                }

                Message::Response(_body) => {
                    // TODO: Transform Errors+ into an error type and return
                    // TODO: Log all others
                }

                message => {
                    return Ok(Some(message));
                }
            }
        }
    }

    pub(super) fn write(&mut self, message: impl Encode) {
        let pos = self.stream.buffer_mut().len();

        message.encode(self.stream.buffer_mut());

        log::trace!(
            "send {:?}",
            bytes::Bytes::from(&self.stream.buffer_mut()[pos..])
        );
    }
}

impl RawConnection for PostgresRawConnection {
    type Backend = Postgres;

    #[inline]
    fn establish(url: &str) -> BoxFuture<Result<Self, Error>> {
        Box::pin(PostgresRawConnection::establish(url))
    }

    #[inline]
    fn finalize<'c>(&'c mut self) -> BoxFuture<'c, Result<(), Error>> {
        Box::pin(self.finalize())
    }

    fn execute<'c>(
        &'c mut self,
        query: &str,
        params: PostgresQueryParameters,
    ) -> BoxFuture<'c, Result<u64, Error>> {
        finish(self, query, params, 0);

        Box::pin(execute::execute(self))
    }

    fn fetch<'c>(
        &'c mut self,
        query: &str,
        params: PostgresQueryParameters,
    ) -> BoxStream<'c, Result<PostgresRow, Error>> {
        finish(self, query, params, 0);

        Box::pin(fetch::fetch(self))
    }

    fn fetch_optional<'c>(
        &'c mut self,
        query: &str,
        params: PostgresQueryParameters,
    ) -> BoxFuture<'c, Result<Option<PostgresRow>, Error>> {
        finish(self, query, params, 1);

        Box::pin(fetch_optional::fetch_optional(self))
    }
}

fn finish(
    conn: &mut PostgresRawConnection,
    query: &str,
    params: PostgresQueryParameters,
    limit: i32,
) {
    conn.write(protocol::Parse {
        portal: "",
        query,
        param_types: &*params.types,
    });

    conn.write(protocol::Bind {
        portal: "",
        statement: "",
        formats: &[1], // [BINARY]
        // TODO: Early error if there is more than i16
        values_len: params.types.len() as i16,
        values: &*params.buf,
        result_formats: &[1], // [BINARY]
    });

    // TODO: Make limit be 1 for fetch_optional
    conn.write(protocol::Execute { portal: "", limit });

    conn.write(protocol::Sync);
}
