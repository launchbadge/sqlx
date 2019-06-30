use bytes::{BufMut, BytesMut};
use futures::{
    io::{AsyncRead, AsyncWriteExt},
    task::{Context, Poll},
    Stream,
};
use runtime::net::TcpStream;
use sqlx_core::ConnectOptions;
use sqlx_postgres_protocol::{Encode, Message, Terminate};
use std::{fmt::Debug, io, pin::Pin};

mod establish;
mod query;

pub struct Connection {
    stream: Framed<TcpStream>,

    // Buffer used when serializing outgoing messages
    // FIXME: Use BytesMut
    wbuf: Vec<u8>,

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
            stream: Framed::new(stream),
            process_id: 0,
            secret_key: 0,
        };

        establish::establish(&mut conn, options).await?;

        Ok(conn)
    }

    pub async fn execute<'a: 'b, 'b>(&'a mut self, query: &'b str) -> io::Result<()> {
        query::query(self, query).await
    }

    pub async fn close(mut self) -> io::Result<()> {
        self.send(Terminate).await?;
        self.stream.inner.close().await?;

        Ok(())
    }

    // Send client message to the server
    async fn send<T>(&mut self, message: T) -> io::Result<()>
    where
        T: Encode + Debug,
    {
        self.wbuf.clear();

        log::trace!("send {:?}", message);

        message.encode(&mut self.wbuf)?;

        log::trace!("send buffer {:?}", bytes::Bytes::from(&*self.wbuf));

        self.stream.inner.write_all(&self.wbuf).await?;
        self.stream.inner.flush().await?;

        Ok(())
    }
}

struct Framed<S> {
    inner: S,
    readable: bool,
    eof: bool,
    buffer: BytesMut,
}

impl<S> Framed<S> {
    fn new(stream: S) -> Self {
        Self {
            readable: false,
            eof: false,
            inner: stream,
            buffer: BytesMut::with_capacity(8 * 1024),
        }
    }
}

impl<S> Stream for Framed<S>
where
    S: AsyncRead + Unpin,
{
    type Item = io::Result<Message>;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context) -> Poll<Option<Self::Item>> {
        let self_ = Pin::get_mut(self);

        loop {
            if self_.readable {
                if self_.eof {
                    return Poll::Ready(None);
                }

                loop {
                    log::trace!("recv buffer {:?}", self_.buffer);

                    let message = Message::decode(&mut self_.buffer)?;

                    if log::log_enabled!(log::Level::Trace) {
                        if let Some(message) = &message {
                            log::trace!("recv {:?}", message);
                        }
                    }

                    match message {
                        Some(Message::ParameterStatus(_body)) => {
                            // TODO: Not sure what to do with these but ignore
                        }

                        Some(Message::Response(_body)) => {
                            // TODO: Handle notices and errors
                        }

                        Some(message) => {
                            return Poll::Ready(Some(Ok(message)));
                        }

                        None => {
                            self_.readable = false;
                            break;
                        }
                    }
                }
            }

            self_.buffer.reserve(32);

            let n = unsafe {
                let b = self_.buffer.bytes_mut();

                self_.inner.initializer().initialize(b);

                let n = match Pin::new(&mut self_.inner).poll_read(cx, b)? {
                    Poll::Ready(cnt) => cnt,
                    Poll::Pending => {
                        return Poll::Pending;
                    }
                };

                self_.buffer.advance_mut(n);

                n
            };

            if n == 0 {
                self_.eof = true;
            }

            self_.readable = true;
        }
    }
}
