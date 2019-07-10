use super::protocol::decode::Decoder;
use crate::protocol::{
    deserialize::Deserialize,
    encode::Encoder,
    packets::{com_ping::ComPing, com_quit::ComQuit, ok::OkPacket},
    serialize::Serialize,
    server::Message as ServerMessage,
    types::{Capabilities, ServerStatusFlag},
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
    pub encoder: Encoder,

    // MariaDB Connection ID
    pub connection_id: i32,

    // Sequence Number
    pub seq_no: u8,

    // Server Capabilities
    pub capabilities: Capabilities,

    // Server status
    pub status: ServerStatusFlag,
}

impl Connection {
    #[cfg(test)]
    pub async fn mock() -> Self {
        let stream: Framed = Framed::new(TcpStream::connect(("127.0.0.1", "3306")).await?);
        let mut conn: Connection = Self {
            stream,
            encoder: Encoder::new(1024),
            connection_id: -1,
            seq_no: 1,
            capabilities: Capabilities::default(),
            status: ServerStatusFlag::default(),
        };

        Ok(conn)
    }

    pub async fn establish(options: ConnectOptions<'static>) -> Result<Self, Error> {
        let stream: Framed = Framed::new(TcpStream::connect((options.host, options.port)).await?);
        let mut conn: Connection = Self {
            stream,
            encoder: Encoder::new(1024),
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
        self.encoder.clear();
        self.encoder.alloc_packet_header();
        self.encoder.seq_no(self.seq_no);
        message.serialize(self)?;
        self.encoder.encode_length();

        self.stream.inner.write_all(&self.encoder.buf).await?;
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
        let buf = self.stream.next_bytes().await?;
        OkPacket::deserialize(self, &mut Decoder::new(&buf))?;

        Ok(())
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
                    self.stream.inner.initializer().initialize(&mut rbuf[len..]);
                }
            }

            let bytes_read = self.stream.inner.read(&mut rbuf[len..]).await?;

            if bytes_read > 0 {
                len += bytes_read;
            } else {
                // Read 0 bytes from the server; end-of-stream
                break;
            }

            while len > 0 {
                let size = rbuf.len();
                let message = ServerMessage::deserialize(self, &mut Decoder::new(&rbuf.as_ref().into()))?;
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
}
