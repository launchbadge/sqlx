use crate::{
    mariadb::{
        protocol::encode, Capabilities, ComInitDb, ComPing, ComQuery, ComQuit, ComStmtPrepare,
        ComStmtPrepareResp, DeContext, Decode, Decoder, Encode, ErrPacket, OkPacket, PacketHeader,
        ProtocolType, ResultSet, ServerStatusFlag,
    },
    options::ConnectOptions,
};
use byteorder::{ByteOrder, LittleEndian};
use bytes::{Bytes, BytesMut};
use core::convert::TryFrom;
use failure::Error;
use futures::{
    io::{AsyncRead},
    prelude::*,
};
use tokio::{
    io::{AsyncReadExt, AsyncWrite, AsyncWriteExt},
    net::TcpStream,
};
use std::net::{SocketAddr, IpAddr, Ipv4Addr};
use url::Url;
use bytes::BufMut;

mod establish;

pub struct Connection {
    pub stream: Framed,

    // Buffer used when serializing outgoing messages
    pub wbuf: Vec<u8>,

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
            status: ServerStatusFlag::SERVER_STATUS_IN_TRANS,
        }
    }

    #[cfg(test)]
    pub fn with_eof() -> Self {
        ConnContext {
            connection_id: 0,
            seq_no: 2,
            last_seq_no: 0,
            capabilities: Capabilities::CLIENT_PROTOCOL_41 | Capabilities::CLIENT_DEPRECATE_EOF,
            status: ServerStatusFlag::SERVER_STATUS_IN_TRANS,
        }
    }
}

impl Connection {
    pub async fn establish(url: &str) -> Result<Self, Error> {
        // TODO: Handle errors
        let url = Url::parse(url).unwrap();

        let host = url.host_str().unwrap_or("localhost");
        let port = url.port().unwrap_or(3306);

        // FIXME: handle errors
        let host: IpAddr = host.parse().unwrap();
        let addr: SocketAddr = (host, port).into();
        let stream: Framed = Framed::new(TcpStream::connect(&addr).await?);
        let mut conn: Connection = Self {
            stream,
            wbuf: Vec::with_capacity(1024),
            context: ConnContext {
                connection_id: -1,
                seq_no: 1,
                last_seq_no: 0,
                capabilities: Capabilities::CLIENT_PROTOCOL_41,
                status: ServerStatusFlag::default(),
            },
        };

        establish::establish(&mut conn, url).await?;

        Ok(conn)
    }

    pub async fn send<S>(&mut self, message: S) -> Result<(), Error>
    where
        S: Encode,
    {
        self.wbuf.clear();

        message.encode(&mut self.wbuf, &mut self.context)?;

        self.stream.inner.write_all(&self.wbuf).await?;

        Ok(())
    }

    pub async fn quit(&mut self) -> Result<(), Error> {
        self.send(ComQuit()).await?;

        Ok(())
    }

    pub async fn query<'a>(
        &'a mut self,
        sql_statement: &'a str,
    ) -> Result<Option<ResultSet>, Error> {
        self.send(ComQuery {
            sql_statement: bytes::Bytes::from(sql_statement),
        })
        .await?;

        let mut ctx = DeContext::with_stream(&mut self.context, &mut self.stream);
        ctx.next_packet().await?;

        match ctx.decoder.peek_tag() {
            0xFF => Err(ErrPacket::decode(&mut ctx)?.into()),
            0x00 => {
                OkPacket::decode(&mut ctx)?;
                Ok(None)
            }
            0xFB => unimplemented!(),
            _ => Ok(Some(ResultSet::deserialize(ctx, ProtocolType::Text).await?)),
        }
    }

    pub async fn select_db<'a>(&'a mut self, db: &'a str) -> Result<(), Error> {
        self.send(ComInitDb {
            schema_name: bytes::Bytes::from(db),
        })
        .await?;

        let mut ctx = DeContext::new(&mut self.context, self.stream.next_packet().await?);
        match ctx.decoder.peek_tag() {
            0xFF => {
                ErrPacket::decode(&mut ctx)?;
            }
            0x00 => {
                OkPacket::decode(&mut ctx)?;
            }
            _ => failure::bail!("Did not receive an ErrPacket nor OkPacket when one was expected"),
        }

        Ok(())
    }

    pub async fn ping(&mut self) -> Result<(), Error> {
        self.send(ComPing()).await?;

        // Ping response must be an OkPacket
        OkPacket::decode(&mut DeContext::new(
            &mut self.context,
            self.stream.next_packet().await?,
        ))?;

        Ok(())
    }

    pub async fn prepare(&mut self, query: &str) -> Result<ComStmtPrepareResp, Error> {
        self.send(ComStmtPrepare {
            statement: Bytes::from(query),
        })
        .await?;

        let mut ctx = DeContext::with_stream(&mut self.context, &mut self.stream);
        ctx.next_packet().await?;
        Ok(ComStmtPrepareResp::deserialize(ctx).await?)
    }
}

pub struct Framed {
    inner: TcpStream,
    buf: BytesMut,
    index: usize,
}

impl Framed {
    fn new(stream: TcpStream) -> Self {
        Self {
            inner: stream,
            buf: BytesMut::with_capacity(8 * 1024),
            index: 0,
        }
    }

    unsafe fn reserve(&mut self, size: usize) {
        self.buf.reserve(size);

        unsafe { self.buf.set_len(self.buf.capacity()); }

        unsafe { self.buf.advance_mut(size) }
    }

    pub async fn next_packet(&mut self) -> Result<Bytes, Error> {
        let mut packet_headers: Vec<PacketHeader> = Vec::new();

        loop {
            println!("BUF: {:?}: ", self.buf);
            // If we don't have a packet header or the last packet header had a length of
            // 0xFF_FF_FF (the max possible length); then we must continue receiving packets
            // because the entire message hasn't been received.
            // After this operation we know that packet_headers.last() *SHOULD* always return valid data,
            // so the the use of packet_headers.last().unwrap() is allowed.
            // TODO: Stitch packets together by removing the length and seq_no from in-between packet definitions.
            if let Some(packet_header) = packet_headers.last() {
                if packet_header.length as usize == encode::U24_MAX {
                    packet_headers.push(PacketHeader::try_from(&self.buf[self.index..])?);
                }
            } else if self.buf.len() > 4 {
                match PacketHeader::try_from(&self.buf[0..]) {
                    Ok(v) => packet_headers.push(v),
                    Err(_) => {}
                }
            }

            if let Some(packet_header) = packet_headers.last() {
                if packet_header.combined_length() > self.buf.len() {
                    unsafe { self.reserve(packet_header.combined_length() - self.buf.len()); }
                }
            } else if self.buf.len() == self.index {
                unsafe { self.reserve(32); }
            }

            // If we have a packet_header and the amount of currently read bytes (len) is less than
            // the specified length inside packet_header, then we can continue reading to self.buf.
            // Else if the total number of bytes read is equal to packet_header then we will
            // return self.buf from 0 to self.index as it should contain the entire packet.
            let bytes_read;

            if let Some(packet_header) = packet_headers.last() {
                if packet_header.combined_length() > self.index {
                    bytes_read = self.inner.read(&mut self.buf[self.index..]).await?;
                } else {
                    // Get the packet from the buffer, reset index, and return packet
                    let packet = self.buf.split_to(packet_header.combined_length()).freeze();
                    self.index -= packet.len();
                    return Ok(packet);
                }
            } else {
                bytes_read = self.inner.read(&mut self.buf[self.index..]).await?;
            }

            if bytes_read > 0 {
                self.index += bytes_read;
                // If we have read less than 4 bytes, and we don't already have a packet_header
                // we must try to read again. The packet_header is always present and is 4 bytes long.
                if bytes_read < 4 && packet_headers.len() == 0 {
                    continue;
                }
            } else {
                // Read 0 bytes from the server; end-of-stream
                panic!("Cannot read 0 bytes from stream");
            }
        }
    }
}
