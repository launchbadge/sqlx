use crate::protocol::{
    client::Serialize,
    server::Message as ServerMessage,
    server::InitialHandshakePacket,
    server::Deserialize
};
use bytes::BytesMut;
use futures::{
    channel::mpsc,
    io::{AsyncRead, AsyncReadExt, AsyncWriteExt, ReadHalf, WriteHalf},
    SinkExt, StreamExt,
};
use mason_core::ConnectOptions;
use runtime::{net::TcpStream, task::JoinHandle};
use std::io;
use failure::Error;
use failure::err_msg;
use byteorder::{ByteOrder, LittleEndian};

mod establish;
// mod query;

pub struct Connection {
    writer: WriteHalf<TcpStream>,
    incoming: mpsc::UnboundedReceiver<ServerMessage>,

    // Buffer used when serializing outgoing messages
    wbuf: Vec<u8>,

    // Handle to coroutine reading messages from the stream
    receiver: JoinHandle<Result<(), Error>>,

    // MariaDB Connection ID
    connection_id: i32,
}

impl Connection {
    pub async fn establish(options: ConnectOptions<'_>) -> Result<Self, Error> {
        let stream = TcpStream::connect((options.host, options.port)).await?;
        let (reader, writer) = stream.split();
        let (tx, rx) = mpsc::unbounded();
        let receiver: JoinHandle<Result<(), Error>> = runtime::spawn(receiver(reader, tx));
        let mut conn = Self {
            wbuf: Vec::with_capacity(1024),
            writer,
            receiver,
            incoming: rx,
            connection_id: -1,
        };

        establish::establish(&mut conn, options).await?;

        Ok(conn)
    }

    async fn send<S>(&mut self, message: S) -> Result<(), Error>
    where
        S: Serialize,
    {
        self.wbuf.clear();

        message.serialize(&mut self.wbuf);

        self.writer.write_all(&self.wbuf).await?;
        self.writer.flush().await?;

        Ok(())
    }
}

async fn receiver(
    mut reader: ReadHalf<TcpStream>,
    mut sender: mpsc::UnboundedSender<ServerMessage>,
) -> Result<(), Error> {
    let mut rbuf = BytesMut::with_capacity(0);
    let mut len = 0;
    let mut first_packet = true;

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
        let bytes_read = reader.read(&mut rbuf[len..]).await?;

        if bytes_read > 0 {
            len += bytes_read;
        } else {
            // Read 0 bytes from the server; end-of-stream
            break;
        }

        while len > 0 {
            let size = rbuf.len();
            println!("Buffer: {:?}", rbuf);
            let message = if first_packet {
                println!("init");
                ServerMessage::init(&mut rbuf)?
            } else {
                println!("deser");
                ServerMessage::deserialize(&mut rbuf)?
            };
            println!("Message: {:?}", message);
            len -= size - rbuf.len();

            if let Some(message) = message {
                first_packet = false;
                sender.send(message).await.unwrap();
            } else {
                // Did not receive enough bytes to
                // deserialize a complete message
                break;
            }

        }


    }

    Ok(())
}
