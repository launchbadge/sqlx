use super::protocol::{Encode, Message, Terminate};
use crate::ConnectOptions;
use bytes::{BufMut, BytesMut};
use futures::{
    io::{AsyncReadExt, AsyncWrite, AsyncWriteExt},
    ready,
    task::{Context, Poll},
    Future,
};
use runtime::net::TcpStream;
use std::{fmt::Debug, io, pin::Pin};

mod establish;
mod execute;
mod get;
mod prepare;
mod select;

pub struct Connection {
    pub(super) stream: TcpStream,

    // Do we think that there is data in the read buffer to be decoded
    stream_readable: bool,

    // Have we reached end-of-file (been disconnected)
    stream_eof: bool,

    // Buffer used when sending outgoing messages
    wbuf: Vec<u8>,

    // Buffer used when reading incoming messages
    // TODO: Evaluate if we _really_ want to use BytesMut here
    rbuf: BytesMut,

    // Process ID of the Backend
    process_id: u32,

    // Backend-unique key to use to send a cancel query message to the server
    secret_key: u32,
}

impl Connection {
    pub async fn establish(options: ConnectOptions<'_>) -> io::Result<Self> {
        let stream = TcpStream::connect((options.host, options.port)).await?;
        let mut conn = Self {
            wbuf: Vec::with_capacity(1024),
            rbuf: BytesMut::with_capacity(1024 * 8),
            stream,
            stream_readable: false,
            stream_eof: false,
            process_id: 0,
            secret_key: 0,
        };

        establish::establish(&mut conn, options).await?;

        Ok(conn)
    }

    pub fn prepare(&mut self, query: &str) -> prepare::Prepare {
        prepare::prepare(self, query)
    }

    pub async fn close(mut self) -> io::Result<()> {
        self.send(Terminate);
        self.flush().await?;
        self.stream.close().await?;

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

    fn send<T>(&mut self, message: T)
    where
        T: Encode + Debug,
    {
        log::trace!("encode {:?}", message);

        // TODO: Encoding should not be fallible
        message.encode(&mut self.wbuf).unwrap();
    }

    async fn flush(&mut self) -> io::Result<()> {
        // TODO: Find some other way to print a Vec<u8> as an ASCII escaped string
        log::trace!("send {:?}", bytes::Bytes::from(&*self.wbuf));

        WriteAllVec::new(&mut self.stream, &mut self.wbuf).await?;

        self.stream.flush().await?;

        Ok(())
    }
}

// Derived from: https://rust-lang-nursery.github.io/futures-api-docs/0.3.0-alpha.16/src/futures_util/io/write_all.rs.html#10-13
// With alterations to be more efficient if we're writing from a mutable vector
// that we can erase

// TODO: Move to Core under 'sqlx_core::io' perhaps?
// TODO: Perhaps the futures project wants this?

pub struct WriteAllVec<'a, W: ?Sized + Unpin> {
    writer: &'a mut W,
    buf: &'a mut Vec<u8>,
}

impl<W: ?Sized + Unpin> Unpin for WriteAllVec<'_, W> {}

impl<'a, W: AsyncWrite + ?Sized + Unpin> WriteAllVec<'a, W> {
    pub(super) fn new(writer: &'a mut W, buf: &'a mut Vec<u8>) -> Self {
        WriteAllVec { writer, buf }
    }
}

impl<W: AsyncWrite + ?Sized + Unpin> Future for WriteAllVec<'_, W> {
    type Output = io::Result<()>;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<io::Result<()>> {
        let this = &mut *self;

        while !this.buf.is_empty() {
            let n = ready!(Pin::new(&mut this.writer).poll_write(cx, this.buf))?;

            this.buf.truncate(this.buf.len() - n);

            if n == 0 {
                return Poll::Ready(Err(io::ErrorKind::WriteZero.into()));
            }
        }

        Poll::Ready(Ok(()))
    }
}
