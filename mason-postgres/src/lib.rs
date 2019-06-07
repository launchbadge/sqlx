#![feature(non_exhaustive, async_await)]
#![allow(clippy::needless_lifetimes)]

use crate::protocol::{client, server};
use bytes::BytesMut;
use futures::{
    channel::mpsc,
    io::{AsyncRead, AsyncReadExt, AsyncWriteExt, WriteHalf},
    SinkExt, StreamExt,
};
use runtime::{net::TcpStream, task::JoinHandle};
use std::io;

pub mod protocol;

pub struct Connection {
    buf: Vec<u8>,
    writer: WriteHalf<TcpStream>,
    incoming: mpsc::Receiver<server::Message>,
    reader: Option<JoinHandle<io::Result<()>>>,
}

impl Connection {
    pub async fn open(address: &str) -> io::Result<Self> {
        let stream = TcpStream::connect(address).await?;
        let (mut reader, writer) = stream.split();

        // FIXME: What's a good buffer size here?
        let (mut tx, rx) = mpsc::channel(1024);

        let reader = runtime::spawn(async move {
            let mut buf = BytesMut::with_capacity(0);
            let mut len = 0;

            'reader: loop {
                if len == buf.len() {
                    unsafe {
                        buf.reserve(32);
                        buf.set_len(buf.capacity());
                        reader.initializer().initialize(&mut buf[len..]);
                    }
                }

                let num = reader.read(&mut buf[len..]).await?;
                if num > 0 {
                    len += num;
                }

                while len > 0 && !buf.is_empty() {
                    let size = buf.len();
                    let msg = server::Message::deserialize(&mut buf)?;

                    let removed = size - buf.len();
                    len -= removed;

                    match msg {
                        Some(server::Message::ParameterStatus(body)) => {
                            // FIXME: Proper log
                            log::info!("{:?}", body);
                        }

                        Some(msg) => {
                            tx.send(msg).await.unwrap();
                        }

                        None => {
                            // We have _some_ amount of data but not enough to
                            // deserialize anything
                            break;
                        }
                    }
                }

                // FIXME: This probably doesn't make sense
                if len == 0 && !buf.is_empty() {
                    // Hit end-of-stream
                    break 'reader;
                }
            }

            Ok(())
        });

        Ok(Self {
            // FIXME: What's a good buffer size here?
            buf: Vec::with_capacity(1024),
            writer,
            reader: Some(reader),
            incoming: rx,
        })
    }

    pub async fn startup<'a, 'b: 'a>(
        &'a mut self,
        user: &'b str,
        _password: &'b str,
        database: &'b str,
    ) -> io::Result<()> {
        let params = [("user", user), ("database", database)];

        let msg = client::StartupMessage { params: &params };
        msg.serialize(&mut self.buf);

        self.writer.write_all(&self.buf).await?;
        self.buf.clear();

        self.writer.flush().await?;

        // FIXME: We _actually_ want to wait until ReadyForQuery, ErrorResponse, AuthX, etc.

        while let Some(message) = self.incoming.next().await {
            match message {
                server::Message::AuthenticationOk => {
                    // Do nothing; server is just telling us "you're in"
                }

                server::Message::ReadyForQuery(_) => {
                    // Good to go
                    break;
                }

                _ => {}
            }
        }

        Ok(())
    }

    pub async fn terminate(&mut self) -> io::Result<()> {
        let msg = client::Terminate {};
        msg.serialize(&mut self.buf);

        self.writer.write_all(&self.buf).await?;
        self.buf.clear();

        self.writer.flush().await?;
        self.writer.close().await?;

        if let Some(reader) = self.reader.take() {
            reader.await?;
        }

        Ok(())
    }
}
