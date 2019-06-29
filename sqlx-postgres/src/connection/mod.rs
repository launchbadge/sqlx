use bytes::BytesMut;
use futures::{
    channel::mpsc,
    io::{AsyncRead, AsyncReadExt, AsyncWriteExt, ReadHalf, WriteHalf},
    SinkExt,
};
use runtime::{net::TcpStream, task::JoinHandle};
use sqlx_core::ConnectOptions;
use sqlx_postgres_protocol::{Encode, Message, Terminate};
use std::io;

mod establish;
// mod query;

pub struct Connection {
    writer: WriteHalf<TcpStream>,
    incoming: mpsc::UnboundedReceiver<Message>,

    // Buffer used when serializing outgoing messages
    // FIXME: Use BytesMut
    wbuf: Vec<u8>,

    // Handle to coroutine reading messages from the stream
    receiver: JoinHandle<io::Result<()>>,

    // Process ID of the Backend
    process_id: u32,

    // Backend-unique key to use to send a cancel query message to the server
    secret_key: u32,
}

impl Connection {
    pub async fn establish(options: ConnectOptions<'_>) -> io::Result<Self> {
        let stream = TcpStream::connect((options.host, options.port)).await?;
        let (reader, writer) = stream.split();
        let (tx, rx) = mpsc::unbounded();
        let receiver = runtime::spawn(receiver(reader, tx));
        let mut conn = Self {
            wbuf: Vec::with_capacity(1024),
            writer,
            receiver,
            incoming: rx,
            process_id: 0,
            secret_key: 0,
        };

        establish::establish(&mut conn, options).await?;

        Ok(conn)
    }

    // pub async fn execute<'a, 'b: 'a>(&'a mut self, query: &'b str) -> io::Result<()> {
    //     query::query(self, query).await
    // }

    pub async fn close(mut self) -> io::Result<()> {
        self.send(Terminate).await?;
        self.writer.close().await?;
        self.receiver.await?;

        Ok(())
    }

    // Send client message to the server
    async fn send<T>(&mut self, message: T) -> io::Result<()>
    where
        T: Encode,
    {
        self.wbuf.clear();

        message.encode(&mut self.wbuf)?;

        log::trace!("sending: {:?}", bytes::Bytes::from(self.wbuf.clone()));

        self.writer.write_all(&self.wbuf).await?;
        self.writer.flush().await?;

        Ok(())
    }
}

async fn receiver(
    mut reader: ReadHalf<TcpStream>,
    mut sender: mpsc::UnboundedSender<Message>,
) -> io::Result<()> {
    let mut rbuf = BytesMut::with_capacity(1024);
    let mut len = 0;

    loop {
        // This uses an adaptive system to extend the vector when it fills. We want to
        // avoid paying to allocate and zero a huge chunk of memory if the reader only
        // has 4 bytes while still making large reads if the reader does have a ton
        // of data to return.

        // See: https://github.com/rust-lang-nursery/futures-rs/blob/master/futures-util/src/io/read_to_end.rs#L50-L54

        if len == rbuf.len() {
            rbuf.reserve(32);

            unsafe {
                // Set length to the capacity and efficiently
                // zero-out the memory
                rbuf.set_len(rbuf.capacity());
                reader.initializer().initialize(&mut rbuf[len..]);
            }
        }

        // TODO: Need a select! on a channel that I can trigger to cancel this
        let cnt = reader.read(&mut rbuf[len..]).await?;

        if cnt > 0 {
            len += cnt;
        } else {
            // Read 0 bytes from the server; end-of-stream
            break;
        }

        while len > 0 {
            let size = rbuf.len();
            let message = Message::decode(&mut rbuf)?;
            len -= size - rbuf.len();

            match message {
                Some(Message::ParameterStatus(body)) => {
                    log::debug!("parameter: {} = {}", body.name(), body.value());
                }

                Some(Message::Response(body)) => {
                    log::warn!("response: {:?}", body);
                }

                Some(message) => {
                    sender.send(message).await.unwrap();
                }

                None => {
                    // Did not receive enough bytes to
                    // decode a complete message
                    break;
                }
            }
        }
    }

    Ok(())
}
