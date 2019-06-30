use crate::protocol::{
    client::Serialize,
    client::ComQuit,
    client::ComPing,
    server::Message as ServerMessage,
    server::ServerStatusFlag,
    server::Capabilities,
    server::InitialHandshakePacket,
    server::Deserialize
};
use bytes::BytesMut;
use futures::{
    io::{AsyncRead, AsyncWriteExt},
    task::{Context, Poll},
    Stream,
};
use futures::prelude::*;
use mason_core::ConnectOptions;
use runtime::{net::TcpStream, task::JoinHandle};
use std::io;
use failure::Error;
use failure::err_msg;
use byteorder::{ByteOrder, LittleEndian, WriteBytesExt};
use crate::protocol::serialize::serialize_length;
use bytes::BufMut;
use bytes::Bytes;

mod establish;
// mod query;

pub struct Connection {
    stream: Framed,

    // Buffer used when serializing outgoing messages
    wbuf: BytesMut,

    // MariaDB Connection ID
    connection_id: i32,

    // Sequence Number
    seq_no: u8,

    // Server Capabilities
    capabilities: Capabilities,

    // Server status
    status: ServerStatusFlag,
}

impl Connection {
    pub async fn establish(options: ConnectOptions<'static>) -> Result<Self, Error> {
        let stream: Framed = Framed::new(TcpStream::connect((options.host, options.port)).await?);
        let mut conn = Self {
            stream,
            wbuf: BytesMut::with_capacity(1024),
            connection_id: -1,
            seq_no: 1,
            capabilities: Capabilities::default(),
            status: ServerStatusFlag::default(),
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
        self.wbuf[3] = self.seq_no;

        message.serialize(&mut self.wbuf, &self.capabilities)?;
        serialize_length(&mut self.wbuf);

        println!("{:?}", self.wbuf);

        self.stream.inner.write_all(&self.wbuf).await?;
        self.stream.inner.flush().await?;

        Ok(())
    }

    async fn quit(&mut self) -> Result<(), Error> {
        self.send(ComQuit()).await?;

        Ok(())
    }

    async fn ping(&mut self) -> Result<(), Error> {
        self.seq_no = 0;
        self.send(ComPing()).await?;

        match self.stream.next().await? {
            Some(ServerMessage::OkPacket(message)) => {
                println!("{:?}", message);
                self.seq_no = message.seq_no;
                Ok(())
            }

            Some(ServerMessage::ErrPacket(message)) => {
                Err(err_msg(format!("{:?}", message)))
            }

            Some(message) => {
                panic!("Did not receive OkPacket nor ErrPacket");
            }

            None => {
                panic!("Did not recieve packet");
            }
        }
    }
}

struct Framed {
    inner: TcpStream,
    readable: bool,
    eof: bool,
    buffer: BytesMut,
}

impl Framed {
    fn new(stream: TcpStream) -> Self {
        Self {
            readable: false,
            eof: false,
            inner: stream,
            buffer: BytesMut::with_capacity(8 * 1024),
        }
    }

    async fn next_bytes(&mut self) -> Result<Bytes, Error> {
        let mut rbuf = BytesMut::with_capacity(0);
        let mut len = 0;
        let mut packet_len: u32 = 0;

        loop {
            if len == rbuf.len() {
                rbuf.reserve(32);

                unsafe {
                    // Set length to the capacity and efficiently
                    // zero-out the memory
                    rbuf.set_len(rbuf.capacity());
                    self.inner.initializer().initialize(&mut rbuf[len..]);
                }
            }

            let bytes_read = self.inner.read(&mut rbuf[len..]).await?;

            if bytes_read > 0 {
                len += bytes_read;
            } else {
                // Read 0 bytes from the server; end-of-stream
                return Ok(Bytes::new());
            }

            println!("buf len: {:?}", rbuf);

            if len > 0 && packet_len == 0 {
                packet_len = LittleEndian::read_u24(&rbuf[0..]);
            }

            // Loop until the length of the buffer is the length of the packet
            if packet_len as usize > len {
                continue;
            } else {
                return Ok(rbuf.freeze());
            }
        }
    }

    async fn next(&mut self) -> Result<Option<ServerMessage>, Error> {
        let mut rbuf = BytesMut::with_capacity(0);
        let mut len = 0;

        loop {
            if len == rbuf.len() {
                rbuf.reserve(32);

                unsafe {
                    // Set length to the capacity and efficiently
                    // zero-out the memory
                    rbuf.set_len(rbuf.capacity());
                    self.inner.initializer().initialize(&mut rbuf[len..]);
                }
            }

            let bytes_read = self.inner.read(&mut rbuf[len..]).await?;

            if bytes_read > 0 {
                len += bytes_read;
            } else {
                // Read 0 bytes from the server; end-of-stream
                break;
            }

            while len > 0 {
                let size = rbuf.len();
                let message = ServerMessage::deserialize(&mut rbuf)?;
                len -= size - rbuf.len();

                match message {
                    message @ Some(_) => return Ok(message),
                    // Did not receive enough bytes to
                    // deserialize a complete message
                    None => break,
                }
            }
        }

        Err(err_msg("Failed to get next packet"))
    }
}

//async fn receiver(
//    mut reader: ReadHalf<TcpStream>,
//    mut sender: mpsc::UnboundedSender<ServerMessage>,
//) -> Result<(), Error> {
//    let mut rbuf = BytesMut::with_capacity(0);
//    let mut len = 0;
//    let mut first_packet = true;
//
//    loop {
//        // This uses an adaptive system to extend the vector when it fills. We want to
//        // avoid paying to allocate and zero a huge chunk of memory if the reader only
//        // has 4 bytes while still making large reads if the reader does have a ton
//        // of data to return.
//
//        // See: https://github.com/rust-lang-nursery/futures-rs/blob/master/futures-util/src/io/read_to_end.rs#L50-L54
//
//        if len == rbuf.len() {
//            rbuf.reserve(32);
//
//            unsafe {
//                // Set length to the capacity and efficiently
//                // zero-out the memory
//                rbuf.set_len(rbuf.capacity());
//                reader.initializer().initialize(&mut rbuf[len..]);
//            }
//        }
//
//        // TODO: Need a select! on a channel that I can trigger to cancel this
//        let bytes_read = reader.read(&mut rbuf[len..]).await?;
//
//        if bytes_read > 0 {
//            len += bytes_read;
//        } else {
//            // Read 0 bytes from the server; end-of-stream
//            break;
//        }
//
//        while len > 0 {
//            let size = rbuf.len();
//            let message = if first_packet {
//                ServerMessage::init(&mut rbuf)
//            } else {
//                ServerMessage::deserialize(&mut rbuf)
//            }?;
//            len -= size - rbuf.len();
//
//            if let Some(message) = message {
//                first_packet = false;
//                sender.send(message).await.unwrap();
//            } else {
//                // Did not receive enough bytes to
//                // deserialize a complete message
//                break;
//            }
//
//        }
//
//
//    }
//
//    Ok(())
//}
