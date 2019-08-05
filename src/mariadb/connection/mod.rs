use byteorder::{ByteOrder, LittleEndian};
use bytes::{Bytes, BytesMut};
use failure::Error;
use futures::{
    io::{AsyncRead, AsyncWriteExt},
    prelude::*,
};
use runtime::net::TcpStream;
use core::convert::TryFrom;
use crate::{ConnectOptions, mariadb::{protocol::encode, PacketHeader, Decoder, DeContext, Deserialize, Encoder, ComInitDb, ComPing, ComQuery, ComQuit, OkPacket, Serialize, Message, Capabilities, ServerStatusFlag, ComStmtPrepare, ComStmtPrepareResp, ResultSet, ErrPacket}};

mod establish;

pub struct Connection {
    pub stream: Framed,

    // Buffer used when serializing outgoing messages
    pub encoder: Encoder,

    // Context for the connection
    // Explicitly declared to easily send to deserializers
    pub context: ConnContext,
}

#[derive(Debug)]
pub struct ConnContext {
    // MariaDB Connection ID
    pub connection_id: i32,

    // Sequence Number
    pub seq_no: u8,

    // Last sequence number return by MariaDB
    pub last_seq_no: u8,

    // Server Capabilities
    pub capabilities: Capabilities,

    // Server status
    pub status: ServerStatusFlag,
}

impl ConnContext {
    #[cfg(test)]
    pub fn new() -> Self {
        ConnContext {
            connection_id: 0,
            seq_no: 2,
            last_seq_no: 0,
            capabilities: Capabilities::CLIENT_PROTOCOL_41,
            status: ServerStatusFlag::SERVER_STATUS_IN_TRANS
        }
    }

    #[cfg(test)]
    pub fn with_eof() -> Self {
        ConnContext {
            connection_id: 0,
            seq_no: 2,
            last_seq_no: 0,
            capabilities: Capabilities::CLIENT_PROTOCOL_41 | Capabilities::CLIENT_DEPRECATE_EOF,
            status: ServerStatusFlag::SERVER_STATUS_IN_TRANS
        }
    }
}

impl Connection {
    pub async fn establish(options: ConnectOptions<'static>) -> Result<Self, Error> {
        let stream: Framed = Framed::new(TcpStream::connect((options.host, options.port)).await?);
        let mut conn: Connection = Self {
            stream,
            encoder: Encoder::new(1024),
            context: ConnContext {
                connection_id: -1,
                seq_no: 1,
                last_seq_no: 0,
                capabilities: Capabilities::CLIENT_PROTOCOL_41,
                status: ServerStatusFlag::default(),
            },
        };

        establish::establish(&mut conn, options).await?;

        Ok(conn)
    }

    pub async fn send<S>(&mut self, message: S) -> Result<(), Error> where S: Serialize {
        self.encoder.clear();

        message.serialize(&mut self.context, &mut self.encoder)?;

        self.stream.inner.write_all(&self.encoder.buf).await?;
        self.stream.inner.flush().await?;

        Ok(())
    }

    pub async fn quit(&mut self) -> Result<(), Error> {
        self.send(ComQuit()).await?;

        Ok(())
    }

    pub async fn query<'a>(&'a mut self, sql_statement: &'a str) -> Result<Option<ResultSet>, Error> {
        self.send(ComQuery { sql_statement: bytes::Bytes::from(sql_statement) }).await?;

        let mut ctx = DeContext::with_stream(&mut self.context, &mut self.stream);
        ctx.next_packet().await?;
        match ctx.decoder.peek_tag() {
            0xFF => Err(ErrPacket::deserialize(&mut ctx)?.into()),
            0x00 => {
                OkPacket::deserialize(&mut ctx)?;
                Ok(None)
            },
            0xFB => unimplemented!(),
            _ => {
                Ok(Some(ResultSet::deserialize(ctx).await?))
            }
        }
    }


    pub async fn select_db<'a>(&'a mut self, db: &'a str) -> Result<(), Error> {
        self.send(ComInitDb { schema_name: bytes::Bytes::from(db) }).await?;


        let mut ctx = DeContext::new(&mut self.context, self.stream.next_packet().await?);
        match ctx.decoder.peek_tag() {
            0xFF => {
                ErrPacket::deserialize(&mut ctx)?;
            },
            0x00 => {
                OkPacket::deserialize(&mut ctx)?;
            },
            _ => failure::bail!("Did not receive an ErrPacket nor OkPacket when one was expected"),
        }

        Ok(())
    }

    pub async fn ping(&mut self) -> Result<(), Error> {
        self.send(ComPing()).await?;

        // Ping response must be an OkPacket
        OkPacket::deserialize(&mut DeContext::new(&mut self.context, self.stream.next_packet().await?))?;

        Ok(())
    }

    pub async fn prepare(&mut self, query: &str) -> Result<ComStmtPrepareResp, Error> {
        self.send(ComStmtPrepare {
            statement: Bytes::from(query),
        }).await?;

//        let mut ctx = DeContext::with_stream(&mut self.context, &mut self.stream);
//        ctx.next_packet().await?;
//        ComStmtPrepareResp::deserialize(&mut ctx)
        Ok(ComStmtPrepareResp::default())
    }
}

pub struct Framed {
    inner: TcpStream,
    buf: BytesMut,
}

impl Framed {
    fn new(stream: TcpStream) -> Self {
        Self {
            inner: stream,
            buf: BytesMut::with_capacity(8 * 1024),
        }
    }

    pub async fn next_packet(&mut self) -> Result<Bytes, Error> {
        let mut rbuf = BytesMut::new();
        let mut len = 0usize;
        let mut packet_headers: Vec<PacketHeader> = Vec::new();

        loop {
            if let Some(packet_header) = packet_headers.last() {
                if packet_header.combined_length() > rbuf.len() {
                    let reserve = packet_header.combined_length() - rbuf.len();
                    rbuf.reserve(reserve);

                    unsafe {
                        rbuf.set_len(rbuf.capacity());
                        self.inner.initializer().initialize(&mut rbuf[len..]);
                    }
                }
            } else if rbuf.len() == len {
                rbuf.reserve(32);

                unsafe {
                    rbuf.set_len(rbuf.capacity());
                    self.inner.initializer().initialize(&mut rbuf[len..]);
                }
            }

            // If we have a packet_header and the amount of currently read bytes (len) is less than
            // the specified length inside packet_header, then we can continue reading to rbuf; but
            // only up until packet_header.length.
            // Else if the total number of bytes read is equal to packet_header then we will
            // return rbuf as it should contain the entire packet.
            // Else we read too many bytes -- which shouldn't happen -- and will return an error.
            let bytes_read;

            if let Some(packet_header) = packet_headers.last() {
                if packet_header.combined_length() > len {
                    bytes_read = self.inner.read(&mut rbuf[len..packet_header.combined_length()]).await?;
                } else {
                    return Ok(rbuf.freeze());
                }
            } else {
                // Only read header to make sure that we dont' read the next packets buffer.
                bytes_read = self.inner.read(&mut rbuf[len..len + 4]).await?;
            }

            if bytes_read > 0 {
                len += bytes_read;
                // If we have read less than 4 bytes, and we don't already have a packet_header
                // we must try to read again. The packet_header is always present and is 4 bytes long.
                if bytes_read < 4 && packet_headers.len() == 0 {
                    continue;
                }
            } else {
                // Read 0 bytes from the server; end-of-stream
                return Ok(rbuf.freeze());
            }

            // If we don't have a packet header or the last packet header had a length of
            // 0xFF_FF_FF (the max possible length); then we must continue receiving packets
            // because the entire message hasn't been received.
            // After this operation we know that packet_headers.last() *SHOULD* always return valid data,
            // so the the use of packet_headers.last().unwrap() is allowed.
            // TODO: Stitch packets together by removing the length and seq_no from in-between packet definitions.
            if let Some(packet_header) = packet_headers.last() {
                if packet_header.length as usize == encode::U24_MAX {
                    packet_headers.push(PacketHeader::try_from(&rbuf[0..])?);
                }
            } else {
                packet_headers.push(PacketHeader::try_from(&rbuf[0..])?);
            }
        }
    }
}
