use crate::protocol::{
    client::{ComPing, ComQuit, Serialize},
    encode::encode_length,
    server::{
        Capabilities, Deserialize, Message as ServerMessage,
        ServerStatusFlag, OkPacket
    },
};
use byteorder::{ByteOrder, LittleEndian};
use bytes::{Bytes, BytesMut};
use failure::Error;
use futures::{
    io::{AsyncRead, AsyncWriteExt},
    prelude::*,
};
use mason_core::ConnectOptions;
use runtime::net::TcpStream;

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
        encode_length(&mut self.wbuf);

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

        // Ping response must be an OkPacket
        OkPacket::deserialize(&self.stream.next_bytes().await?)?;

        Ok(())
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
        let mut rbuf = BytesMut::new();
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
        let mut rbuf = BytesMut::new();
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

        Ok(None)
    }
}
