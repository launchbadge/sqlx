use crate::protocol::{
    client::Serialize,
    client::ComQuit,
    client::ComPing,
    server::Message as ServerMessage,
    server::Capabilities,
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
use byteorder::{ByteOrder, LittleEndian, WriteBytesExt};
use crate::protocol::serialize::serialize_length;
use bytes::BufMut;

mod establish;
// mod query;

pub struct Connection {
    writer: WriteHalf<TcpStream>,
    incoming: mpsc::UnboundedReceiver<ServerMessage>,

    // Buffer used when serializing outgoing messages
    wbuf: BytesMut,

    // Handle to coroutine reading messages from the stream
    receiver: JoinHandle<Result<(), Error>>,

    // MariaDB Connection ID
    connection_id: i32,

    // Sequence Number
    sequence_number: u8,

    // Server Capabilities
    server_capabilities: Capabilities,
}

impl Connection {
    pub async fn establish(options: ConnectOptions<'static>) -> Result<Self, Error> {
        let stream = TcpStream::connect((options.host, options.port)).await?;
        let (reader, writer) = stream.split();
        let (tx, rx) = mpsc::unbounded();
        let receiver: JoinHandle<Result<(), Error>> = runtime::spawn(receiver(reader, tx));
        let mut conn = Self {
            wbuf: BytesMut::with_capacity(1024),
            writer,
            receiver,
            incoming: rx,
            connection_id: -1,
            sequence_number: 1,
            server_capabilities: Capabilities::default(),
        };

        establish::establish(&mut conn, options).await?;

        Ok(conn)
    }

    async fn send<S>(&mut self, message: S) -> Result<(), Error>
    where
        S: Serialize,
    {
        self.wbuf.clear();

        /*
            `self.wbuf.write_u32::<LittleEndian>(0_u32);`
            causes compiler to panic
            self.wbuf.write
            rustc 1.37.0-nightly (7cdaffd79 2019-06-05) running on x86_64-unknown-linux-gnu
            https://github.com/rust-lang/rust/issues/62126
        */
        // Reserve space for packet header; Packet Body Length (3 bytes) and sequence number (1 byte)
        self.wbuf.extend_from_slice(&[0; 4]);
        self.wbuf[3] = self.sequence_number;

        message.serialize(&mut self.wbuf, &self.server_capabilities)?;
        serialize_length(&mut self.wbuf);

        println!("{:?}", self.wbuf);

        self.writer.write_all(&self.wbuf).await?;
        self.writer.flush().await?;

        Ok(())
    }

    async fn quit(&mut self) -> Result<(), Error> {
        self.send(ComQuit()).await?;

        Ok(())
    }

    async fn ping(&mut self) -> Result<(), Error> {
        self.sequence_number = 0;
        self.send(ComPing()).await?;

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
            let message = if first_packet {
                ServerMessage::init(&mut rbuf)
            } else {
                ServerMessage::deserialize(&mut rbuf)
            }?;
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
