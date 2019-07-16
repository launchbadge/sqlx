use bytes::{BufMut, BytesMut};
use futures::{
    io::{AsyncRead, AsyncWriteExt},
    ready,
    task::{Context, Poll},
    Stream,
};
use runtime::net::TcpStream;
use sqlx_core::ConnectOptions;
use sqlx_postgres_protocol::{Encode, Message, Terminate};
use std::{fmt::Debug, io, pin::Pin, sync::atomic::AtomicU64};

mod establish;
mod execute;

pub struct Connection {
    pub(super) stream: Framed<TcpStream>,

    // HACK: This is how we currently "name" queries when executing
    statement_index: AtomicU64,

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
            statement_index: AtomicU64::new(0),
        };

        establish::establish(&mut conn, options).await?;

        Ok(conn)
    }

    #[inline]
    pub fn execute<'a>(&'a mut self, query: &'a str) -> execute::Execute<'a> {
        execute::execute(self, query)
    }

    pub async fn close(mut self) -> io::Result<()> {
        self.send(Terminate).await?;
        self.stream.inner.close().await?;

        Ok(())
    }

    async fn send<T>(&mut self, message: T) -> io::Result<()>
    where
        T: Encode + Debug,
    {
        self.wbuf.clear();

        message.encode(&mut self.wbuf)?;

        self.stream.inner.write_all(&self.wbuf).await?;
        self.stream.inner.flush().await?;

        Ok(())
    }
}

pub(super) struct Framed<S> {
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
                    match Message::decode(&mut self_.buffer)? {
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

                let n = ready!(Pin::new(&mut self_.inner).poll_read(cx, b))?;

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
