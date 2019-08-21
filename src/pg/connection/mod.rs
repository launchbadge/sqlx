use super::{
    protocol::{Encode, Message, Terminate},
    Pg, PgRow,
};
use crate::{connection::RawConnection, query::RawQuery};
use bytes::{BufMut, BytesMut};
use futures_core::{future::BoxFuture, stream::BoxStream};
use std::{
    io,
    net::{IpAddr, Shutdown, SocketAddr},
};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::TcpStream,
};
use url::Url;

mod establish;
mod execute;
mod fetch;
mod fetch_optional;

pub struct PgRawConnection {
    stream: TcpStream,

    // Do we think that there is data in the read buffer to be decoded
    stream_readable: bool,

    // Have we reached end-of-file (been disconnected)
    stream_eof: bool,

    // Buffer used when sending outgoing messages
    pub(super) wbuf: Vec<u8>,

    // Buffer used when reading incoming messages
    // TODO: Evaluate if we _really_ want to use BytesMut here
    rbuf: BytesMut,

    // Process ID of the Backend
    process_id: u32,

    // Backend-unique key to use to send a cancel query message to the server
    secret_key: u32,
}

impl PgRawConnection {
    async fn establish(url: &str) -> io::Result<Self> {
        // TODO: Handle errors
        let url = Url::parse(url).unwrap();

        let host = url.host_str().unwrap_or("localhost");
        let port = url.port().unwrap_or(5432);

        // FIXME: handle errors
        let host: IpAddr = host.parse().unwrap();
        let addr: SocketAddr = (host, port).into();

        let stream = TcpStream::connect(&addr).await?;
        let mut conn = Self {
            wbuf: Vec::with_capacity(1024),
            rbuf: BytesMut::with_capacity(1024 * 8),
            stream,
            stream_readable: false,
            stream_eof: false,
            process_id: 0,
            secret_key: 0,
        };

        establish::establish(&mut conn, &url).await?;

        Ok(conn)
    }

    async fn finalize(&mut self) -> io::Result<()> {
        self.write(Terminate);
        self.flush().await?;
        self.stream.shutdown(Shutdown::Both)?;

        Ok(())
    }

    // Wait and return the next message to be received from Postgres.
    async fn receive(&mut self) -> io::Result<Option<Message>> {
        loop {
            if self.stream_eof {
                // Reached end-of-file on a previous read call.
                return Ok(None);
            }

            if self.stream_readable {
                loop {
                    match Message::decode(&mut self.rbuf)? {
                        Some(Message::ParameterStatus(_body)) => {
                            // TODO: not sure what to do with these yet
                        }

                        Some(Message::Response(_body)) => {
                            // TODO: Transform Errors+ into an error type and return
                            // TODO: Log all others
                        }

                        Some(message) => {
                            return Ok(Some(message));
                        }

                        None => {
                            // Not enough data in the read buffer to parse a message
                            self.stream_readable = true;
                            break;
                        }
                    }
                }
            }

            // Ensure there is at least 32-bytes of space available
            // in the read buffer so we can safely detect end-of-file
            self.rbuf.reserve(32);

            // SAFE: Read data in directly to buffer without zero-initializing the data.
            //       Postgres is a self-describing format and the TCP frames encode
            //       length headers. We will never attempt to decode more than we
            //       received.
            let n = self.stream.read(unsafe { self.rbuf.bytes_mut() }).await?;

            // SAFE: After we read in N bytes, we can tell the buffer that it actually
            //       has that many bytes MORE for the decode routines to look at
            unsafe { self.rbuf.advance_mut(n) }

            if n == 0 {
                self.stream_eof = true;
            }

            self.stream_readable = true;
        }
    }

    pub(super) fn write(&mut self, message: impl Encode) {
        message.encode(&mut self.wbuf);
    }

    async fn flush(&mut self) -> io::Result<()> {
        self.stream.write_all(&self.wbuf).await?;
        self.wbuf.clear();

        Ok(())
    }
}

impl RawConnection for PgRawConnection {
    type Backend = Pg;

    #[inline]
    fn establish(url: &str) -> BoxFuture<io::Result<Self>> {
        Box::pin(PgRawConnection::establish(url))
    }

    #[inline]
    fn finalize<'c>(&'c mut self) -> BoxFuture<'c, io::Result<()>> {
        Box::pin(self.finalize())
    }

    fn execute<'c, 'q, Q: 'q>(&'c mut self, query: Q) -> BoxFuture<'c, io::Result<u64>>
    where
        Q: RawQuery<'q, Backend = Self::Backend>,
    {
        query.finish(self);

        Box::pin(execute::execute(self))
    }

    fn fetch<'c, 'q, Q: 'q>(&'c mut self, query: Q) -> BoxStream<'c, io::Result<PgRow>>
    where
        Q: RawQuery<'q, Backend = Self::Backend>,
    {
        query.finish(self);

        Box::pin(fetch::fetch(self))
    }

    fn fetch_optional<'c, 'q, Q: 'q>(
        &'c mut self,
        query: Q,
    ) -> BoxFuture<'c, io::Result<Option<PgRow>>>
    where
        Q: RawQuery<'q, Backend = Self::Backend>,
    {
        query.finish(self);

        Box::pin(fetch_optional::fetch_optional(self))
    }
}
