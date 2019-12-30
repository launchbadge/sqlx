use std::convert::TryInto;
use std::io;

use async_std::net::{Shutdown, TcpStream};
use byteorder::{ByteOrder, LittleEndian};
use futures_core::future::BoxFuture;

use crate::cache::StatementCache;
use crate::connection::Connection;
use crate::executor::Executor;
use crate::io::{Buf, BufMut, BufStream};
use crate::mysql::error::MySqlError;
use crate::mysql::protocol::{
    Capabilities, Decode, Encode, EofPacket, ErrPacket, Handshake, HandshakeResponse, OkPacket,
};
use crate::url::Url;

/// An asynchronous connection to a [MySql] database.
///
/// The connection string expected by [Connection::open] should be a MySQL connection
/// string, as documented at
/// <https://dev.mysql.com/doc/refman/8.0/en/connecting-using-uri-or-key-value-pairs.html#connecting-using-uri>
pub struct MySqlConnection {
    pub(super) stream: BufStream<TcpStream>,

    pub(super) capabilities: Capabilities,

    pub(super) statement_cache: StatementCache<u32>,

    rbuf: Vec<u8>,

    next_seq_no: u8,

    pub(super) ready: bool,
}

impl MySqlConnection {
    pub(super) fn begin_command_phase(&mut self) {
        // At the start of the *command phase*, the sequence ID sent from the client
        // must be 0
        self.next_seq_no = 0;
    }

    pub(super) fn write(&mut self, packet: impl Encode + std::fmt::Debug) {
        let buf = self.stream.buffer_mut();

        // Allocate room for the header that we write after the packet;
        // so, we can get an accurate and cheap measure of packet length

        let header_offset = buf.len();
        buf.advance(4);

        packet.encode(buf, self.capabilities);

        // Determine length of encoded packet
        // and write to allocated header

        let len = buf.len() - header_offset - 4;
        let mut header = &mut buf[header_offset..];

        LittleEndian::write_u32(&mut header, len as u32); // len

        // Take the last sequence number received, if any, and increment by 1
        // If there was no sequence number, we only increment if we split packets
        header[3] = self.next_seq_no;
        self.next_seq_no = self.next_seq_no.wrapping_add(1);
    }

    async fn receive_ok(&mut self) -> crate::Result<OkPacket> {
        let packet = self.receive().await?;
        Ok(match packet[0] {
            0xfe | 0x00 => OkPacket::decode(packet)?,

            0xff => {
                return Err(MySqlError(ErrPacket::decode(packet)?).into());
            }

            id => {
                return Err(protocol_err!(
                    "unexpected packet identifier 0x{:X?} when expecting 0xFE (OK) or 0xFF \
                     (ERR)",
                    id
                )
                .into());
            }
        })
    }

    pub(super) async fn receive_eof(&mut self) -> crate::Result<()> {
        // When (legacy) EOFs are enabled, the fixed number column definitions are further
        // terminated by an EOF packet
        if !self.capabilities.contains(Capabilities::DEPRECATE_EOF) {
            let _eof = EofPacket::decode(self.receive().await?)?;
        }

        Ok(())
    }

    pub(super) async fn receive(&mut self) -> crate::Result<&[u8]> {
        Ok(self
            .try_receive()
            .await?
            .ok_or(io::ErrorKind::UnexpectedEof)?)
    }

    pub(super) async fn try_receive(&mut self) -> crate::Result<Option<&[u8]>> {
        self.rbuf.clear();

        // Read the packet header which contains the length and the sequence number
        // https://dev.mysql.com/doc/dev/mysql-server/8.0.12/page_protocol_basic_packets.html
        // https://mariadb.com/kb/en/library/0-packet/#standard-packet
        let mut header = ret_if_none!(self.stream.peek(4).await?);
        let payload_len = header.get_uint::<LittleEndian>(3)? as usize;
        self.next_seq_no = header.get_u8()?.wrapping_add(1);
        self.stream.consume(4);

        // Read the packet body and copy it into our internal buf
        // We must have a separate buffer around the stream as we can't operate directly
        // on bytes returned from the stream. We have various kinds of payload manipulation
        // that must be handled before decoding.
        let mut payload = ret_if_none!(self.stream.peek(payload_len).await?);
        self.rbuf.extend_from_slice(payload);
        self.stream.consume(payload_len);

        // TODO: Implement packet compression
        // TODO: Implement packet joining

        Ok(Some(&self.rbuf[..payload_len]))
    }
}

impl MySqlConnection {
    // TODO: Authentication ?!
    pub(super) async fn open(url: crate::Result<Url>) -> crate::Result<Self> {
        let url = url?;
        let stream = TcpStream::connect((url.host(), url.port(3306))).await?;

        let mut self_ = Self {
            stream: BufStream::new(stream),
            capabilities: Capabilities::empty(),
            rbuf: Vec::with_capacity(8192),
            next_seq_no: 0,
            statement_cache: StatementCache::new(),
            ready: true,
        };

        // https://dev.mysql.com/doc/dev/mysql-server/8.0.12/page_protocol_connection_phase.html
        // https://mariadb.com/kb/en/connection/

        // First, we receive the Handshake

        let handshake_packet = self_.receive().await?;
        let handshake = Handshake::decode(handshake_packet)?;

        let client_capabilities = Capabilities::PROTOCOL_41
            | Capabilities::IGNORE_SPACE
            | Capabilities::FOUND_ROWS
            | Capabilities::CONNECT_WITH_DB;

        // Fails if [Capabilities::PROTOCOL_41] is not in [server_capabilities]
        self_.capabilities =
            (client_capabilities & handshake.server_capabilities) | Capabilities::PROTOCOL_41;

        // Next we send the response

        self_.write(HandshakeResponse {
            client_collation: 192, // utf8_unicode_ci
            max_packet_size: 1024,
            username: url.username().unwrap_or("root"),
            // TODO: Remove the panic!
            database: url.database().expect("required database"),
            auth_plugin_name: handshake.auth_plugin_name.as_deref(),
            auth_response: None,
        });

        self_.stream.flush().await?;

        let _ok = self_.receive_ok().await?;

        // On connect, we want to establish a modern, Rust-compatible baseline so we
        // tweak connection options to enable UTC for TIMESTAMP, UTF-8 for character types, etc.

        // TODO: Use batch support when we have it to handle the following in one execution

        // https://mariadb.com/kb/en/sql-mode/

        // PIPES_AS_CONCAT - Allows using the pipe character (ASCII 124) as string concatenation operator.
        //                   This means that "A" || "B" can be used in place of CONCAT("A", "B").

        // NO_ENGINE_SUBSTITUTION - If not set, if the available storage engine specified by a CREATE TABLE is
        //                          not available, a warning is given and the default storage
        //                          engine is used instead.

        // NO_ZERO_DATE - Don't allow '0000-00-00'. This is invalid in Rust.

        // NO_ZERO_IN_DATE - Don't allow 'yyyy-00-00'. This is invalid in Rust.

        self_.send("SET sql_mode=(SELECT CONCAT(@@sql_mode, ',PIPES_AS_CONCAT,NO_ENGINE_SUBSTITUTION,NO_ZERO_DATE,NO_ZERO_IN_DATE'))")
            .await?;

        // This allows us to assume that the output from a TIMESTAMP field is UTC

        self_.send("SET time_zone = 'UTC'").await?;

        // https://mathiasbynens.be/notes/mysql-utf8mb4

        self_
            .send("SET NAMES utf8mb4 COLLATE utf8mb4_unicode_ci")
            .await?;

        Ok(self_)
    }

    async fn close(mut self) -> crate::Result<()> {
        self.stream.flush().await?;
        self.stream.stream.shutdown(Shutdown::Both)?;

        Ok(())
    }
}

impl Connection for MySqlConnection {
    fn open<T>(url: T) -> BoxFuture<'static, crate::Result<Self>>
    where
        T: TryInto<Url, Error = crate::Error>,
        Self: Sized,
    {
        Box::pin(MySqlConnection::open(url.try_into()))
    }

    fn close(self) -> BoxFuture<'static, crate::Result<()>> {
        Box::pin(self.close())
    }
}
