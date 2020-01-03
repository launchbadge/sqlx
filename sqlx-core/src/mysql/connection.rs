use std::convert::TryInto;
use std::io;

use async_std::net::{Shutdown, TcpStream};
use byteorder::{ByteOrder, LittleEndian};
use futures_core::future::BoxFuture;
use sha1::Sha1;
use sha2::{Digest, Sha256};

use crate::cache::StatementCache;
use crate::connection::Connection;
use crate::executor::Executor;
use crate::io::{Buf, BufMut, BufStream};
use crate::mysql::error::MySqlError;
use crate::mysql::protocol::{
    AuthPlugin, AuthSwitch, Capabilities, Decode, Encode, EofPacket, ErrPacket, Handshake,
    HandshakeResponse, OkPacket,
};
use crate::mysql::rsa;
use crate::mysql::util::xor_eq;
use crate::url::Url;

// Size before a packet is split
const MAX_PACKET_SIZE: u32 = 1024;

const COLLATE_UTF8MB4_UNICODE_CI: u8 = 224;

/// An asynchronous connection to a [MySql] database.
///
/// The connection string expected by [Connection::open] should be a MySQL connection
/// string, as documented at
/// <https://dev.mysql.com/doc/refman/8.0/en/connecting-using-uri-or-key-value-pairs.html#connecting-using-uri>
pub struct MySqlConnection {
    pub(super) stream: BufStream<TcpStream>,

    // Active capabilities of the client _&_ the server
    pub(super) capabilities: Capabilities,

    // Cache of prepared statements
    //  Query (String) to StatementId to ColumnMap
    pub(super) statement_cache: StatementCache<u32>,

    // Packets are buffered into a second buffer from the stream
    // as we may have compressed or split packets to figure out before
    // decoding
    pub(super) packet: Vec<u8>,
    packet_len: usize,

    // Packets in a command sequence have an incrementing sequence number
    // This number must be 0 at the start of each command
    pub(super) next_seq_no: u8,
}

impl MySqlConnection {
    /// Write the packet to the stream ( do not send to the server )
    pub(crate) fn write(&mut self, packet: impl Encode) {
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

    /// Send the packet to the database server
    pub(crate) async fn send(&mut self, packet: impl Encode) -> crate::Result<()> {
        self.write(packet);
        self.stream.flush().await?;

        Ok(())
    }

    /// Send a [HandshakeResponse] packet to the database server
    pub(crate) async fn send_handshake_response(
        &mut self,
        url: &Url,
        auth_plugin: &AuthPlugin,
        auth_response: &[u8],
    ) -> crate::Result<()> {
        self.send(HandshakeResponse {
            client_collation: COLLATE_UTF8MB4_UNICODE_CI,
            max_packet_size: MAX_PACKET_SIZE,
            username: url.username().unwrap_or("root"),
            database: url.database(),
            auth_plugin,
            auth_response,
        })
        .await
    }

    /// Try to receive a packet from the database server. Returns `None` if the server has sent
    /// no data.
    pub(crate) async fn try_receive(&mut self) -> crate::Result<Option<()>> {
        self.packet.clear();

        // Read the packet header which contains the length and the sequence number
        // https://dev.mysql.com/doc/dev/mysql-server/8.0.12/page_protocol_basic_packets.html
        // https://mariadb.com/kb/en/library/0-packet/#standard-packet
        let mut header = ret_if_none!(self.stream.peek(4).await?);
        self.packet_len = header.get_uint::<LittleEndian>(3)? as usize;
        self.next_seq_no = header.get_u8()?.wrapping_add(1);
        self.stream.consume(4);

        // Read the packet body and copy it into our internal buf
        // We must have a separate buffer around the stream as we can't operate directly
        // on bytes returned from the stream. We have various kinds of payload manipulation
        // that must be handled before decoding.
        let mut payload = ret_if_none!(self.stream.peek(self.packet_len).await?);
        self.packet.extend_from_slice(payload);
        self.stream.consume(self.packet_len);

        // TODO: Implement packet compression
        // TODO: Implement packet joining

        Ok(Some(()))
    }

    /// Receive a complete packet from the database server.
    pub(crate) async fn receive(&mut self) -> crate::Result<&mut Self> {
        self.try_receive()
            .await?
            .ok_or(io::ErrorKind::UnexpectedEof)?;

        Ok(self)
    }

    /// Returns a reference to the most recently received packet data
    #[inline]
    pub(crate) fn packet(&self) -> &[u8] {
        &self.packet[..self.packet_len]
    }

    /// Receive an [EofPacket] if we are supposed to receive them at all.
    pub(crate) async fn receive_eof(&mut self) -> crate::Result<()> {
        // When (legacy) EOFs are enabled, many things are terminated by an EOF packet
        if !self.capabilities.contains(Capabilities::DEPRECATE_EOF) {
            let _eof = EofPacket::decode(self.receive().await?.packet())?;
        }

        Ok(())
    }

    /// Receive a [Handshake] packet. When connecting to the database server, this is immediately
    /// received from the database server.
    pub(crate) async fn receive_handshake(&mut self, url: &Url) -> crate::Result<Handshake> {
        let handshake = Handshake::decode(self.receive().await?.packet())?;

        let mut client_capabilities = Capabilities::PROTOCOL_41
            | Capabilities::IGNORE_SPACE
            | Capabilities::FOUND_ROWS
            | Capabilities::PLUGIN_AUTH;

        if url.database().is_some() {
            client_capabilities |= Capabilities::CONNECT_WITH_DB;
        }

        self.capabilities =
            (client_capabilities & handshake.server_capabilities) | Capabilities::PROTOCOL_41;

        Ok(handshake)
    }

    /// Receives an [OkPacket] from the database server. This is called at the end of
    /// authentication to confirm the established connection.
    pub(crate) fn receive_auth_ok<'a>(
        &'a mut self,
        plugin: &'a AuthPlugin,
        password: &'a str,
        nonce: &'a [u8],
    ) -> BoxFuture<'a, crate::Result<()>> {
        Box::pin(async move {
            self.receive().await?;

            match self.packet[0] {
                0x00 => self.handle_ok().map(drop),
                0xfe => self.handle_auth_switch(password).await,
                0xff => self.handle_err(),

                _ => self.handle_auth_continue(plugin, password, nonce).await,
            }
        })
    }
}

impl MySqlConnection {
    pub(crate) fn handle_ok(&mut self) -> crate::Result<OkPacket> {
        let ok = OkPacket::decode(self.packet())?;

        // An OK signifies the end of the current command sequence
        self.next_seq_no = 0;

        Ok(ok)
    }

    pub(crate) fn handle_err<T>(&mut self) -> crate::Result<T> {
        let err = ErrPacket::decode(self.packet())?;

        // An ERR signifies the end of the current command sequence
        self.next_seq_no = 0;

        Err(MySqlError(err).into())
    }

    pub(crate) fn handle_unexpected_packet<T>(&self, id: u8) -> crate::Result<T> {
        Err(protocol_err!("unexpected packet identifier 0x{:X?}", id).into())
    }

    pub(crate) async fn handle_auth_continue(
        &mut self,
        plugin: &AuthPlugin,
        password: &str,
        nonce: &[u8],
    ) -> crate::Result<()> {
        match plugin {
            AuthPlugin::CachingSha2Password => {
                if self.packet[0] == 1 {
                    match self.packet[1] {
                        // AUTH_OK
                        0x03 => {}

                        // AUTH_CONTINUE
                        0x04 => {
                            // client sends an RSA encrypted password
                            let ct = self.rsa_encrypt(0x02, password, nonce).await?;

                            self.send(&*ct).await?;
                        }

                        auth => {
                            return Err(protocol_err!("unexpected result from 'fast' authentication 0x{:x} when expecting OK (0x03) or CONTINUE (0x04)", auth).into());
                        }
                    }

                    // ends with server sending either OK_Packet or ERR_Packet
                    self.receive_auth_ok(plugin, password, nonce)
                        .await
                        .map(drop)
                } else {
                    return self.handle_unexpected_packet(self.packet[0]);
                }
            }

            // No other supported auth methods will be called through continue
            _ => unreachable!(),
        }
    }

    pub(crate) async fn handle_auth_switch(&mut self, password: &str) -> crate::Result<()> {
        let auth = AuthSwitch::decode(self.packet())?;

        let auth_response = self
            .make_auth_initial_response(&auth.auth_plugin, password, &auth.auth_plugin_data)
            .await?;

        self.send(&*auth_response).await?;

        self.receive_auth_ok(&auth.auth_plugin, password, &auth.auth_plugin_data)
            .await
    }

    pub(crate) async fn make_auth_initial_response(
        &mut self,
        plugin: &AuthPlugin,
        password: &str,
        nonce: &[u8],
    ) -> crate::Result<Vec<u8>> {
        match plugin {
            AuthPlugin::CachingSha2Password | AuthPlugin::MySqlNativePassword => {
                Ok(plugin.scramble(password, nonce))
            }

            AuthPlugin::Sha256Password => {
                // Full RSA exchange and password encrypt up front with no "cache"
                Ok(self.rsa_encrypt(0x01, password, nonce).await?.into_vec())
            }
        }
    }

    pub(crate) async fn rsa_encrypt(
        &mut self,
        public_key_request_id: u8,
        password: &str,
        nonce: &[u8],
    ) -> crate::Result<Box<[u8]>> {
        // https://mariadb.com/kb/en/caching_sha2_password-authentication-plugin/

        // TODO: Handle SSL

        // client sends a public key request
        self.send(&[public_key_request_id][..]).await?;

        // server sends a public key response
        let mut packet = self.receive().await?.packet();
        let rsa_pub_key = &packet[1..];

        // The password string data must be NUL terminated
        // Note: This is not in the documentation that I could find
        let mut pass = password.as_bytes().to_vec();
        pass.push(0);

        xor_eq(&mut pass, nonce);

        // client sends an RSA encrypted password
        rsa::encrypt::<Sha1>(rsa_pub_key, &pass)
    }
}

impl MySqlConnection {
    async fn new(url: &Url) -> crate::Result<Self> {
        let stream = TcpStream::connect((url.host(), url.port(3306))).await?;

        Ok(Self {
            stream: BufStream::new(stream),
            capabilities: Capabilities::empty(),
            packet: Vec::with_capacity(8192),
            packet_len: 0,
            next_seq_no: 0,
            statement_cache: StatementCache::new(),
        })
    }

    async fn initialize(&mut self) -> crate::Result<()> {
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

        // NO_ZERO_IN_DATE - Don't allow 'YYYY-00-00'. This is invalid in Rust.

        // language=MySQL
        self.execute_raw("SET sql_mode=(SELECT CONCAT(@@sql_mode, ',PIPES_AS_CONCAT,NO_ENGINE_SUBSTITUTION,NO_ZERO_DATE,NO_ZERO_IN_DATE'))")
            .await?;

        // This allows us to assume that the output from a TIMESTAMP field is UTC

        // language=MySQL
        self.execute_raw("SET time_zone = 'UTC'").await?;

        // https://mathiasbynens.be/notes/mysql-utf8mb4

        // language=MySQL
        self.execute_raw("SET NAMES utf8mb4 COLLATE utf8mb4_unicode_ci")
            .await?;

        Ok(())
    }
}

impl MySqlConnection {
    pub(super) async fn open(url: crate::Result<Url>) -> crate::Result<Self> {
        let url = url?;
        let mut self_ = Self::new(&url).await?;

        // https://dev.mysql.com/doc/dev/mysql-server/8.0.12/page_protocol_connection_phase.html
        // https://mariadb.com/kb/en/connection/

        // On connect, server immediately sends the handshake
        let handshake = self_.receive_handshake(&url).await?;

        // Pre-generate an auth response by using the auth method in the [Handshake]
        let password = url.password().unwrap_or_default();
        let auth_response = self_
            .make_auth_initial_response(
                &handshake.auth_plugin,
                password,
                &handshake.auth_plugin_data,
            )
            .await?;

        self_
            .send_handshake_response(&url, &handshake.auth_plugin, &auth_response)
            .await?;

        // After sending the handshake response with our assumed auth method the server
        // will send OK, fail, or tell us to change auth methods
        self_
            .receive_auth_ok(
                &handshake.auth_plugin,
                password,
                &handshake.auth_plugin_data,
            )
            .await?;

        // After the connection is established, we initialize by configuring a few
        // connection parameters
        self_.initialize().await?;

        Ok(self_)
    }

    async fn close(mut self) -> crate::Result<()> {
        // TODO: Actually tell MySQL that we're closing

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
