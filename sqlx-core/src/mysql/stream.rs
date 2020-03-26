use std::net::Shutdown;

use byteorder::{ByteOrder, LittleEndian};

use crate::io::{Buf, BufMut, BufStream, MaybeTlsStream};
use crate::mysql::protocol::{Capabilities, Encode, EofPacket, ErrPacket, OkPacket};

use crate::mysql::MySqlError;
use crate::url::Url;

// Size before a packet is split
const MAX_PACKET_SIZE: u32 = 1024;

pub(crate) struct MySqlStream {
    pub(super) stream: BufStream<MaybeTlsStream>,

    // Is the stream ready to send commands
    // Put another way, are we still expecting an EOF or OK packet to terminate
    pub(super) is_ready: bool,

    // Active capabilities
    pub(super) capabilities: Capabilities,

    // Packets in a command sequence have an incrementing sequence number
    // This number must be 0 at the start of each command
    pub(super) seq_no: u8,

    // Packets are buffered into a second buffer from the stream
    // as we may have compressed or split packets to figure out before
    // decoding
    packet_buf: Vec<u8>,
    packet_len: usize,
}

impl MySqlStream {
    pub(super) async fn new(url: &Url) -> crate::Result<Self> {
        let stream = MaybeTlsStream::connect(&url, 3306).await?;

        let mut capabilities = Capabilities::PROTOCOL_41
            | Capabilities::IGNORE_SPACE
            | Capabilities::DEPRECATE_EOF
            | Capabilities::FOUND_ROWS
            | Capabilities::TRANSACTIONS
            | Capabilities::SECURE_CONNECTION
            | Capabilities::PLUGIN_AUTH_LENENC_DATA
            | Capabilities::MULTI_STATEMENTS
            | Capabilities::MULTI_RESULTS
            | Capabilities::PLUGIN_AUTH;

        if url.database().is_some() {
            capabilities |= Capabilities::CONNECT_WITH_DB;
        }

        if cfg!(feature = "tls") {
            capabilities |= Capabilities::SSL;
        }

        Ok(Self {
            capabilities,
            stream: BufStream::new(stream),
            packet_buf: Vec::with_capacity(MAX_PACKET_SIZE as usize),
            packet_len: 0,
            seq_no: 0,
            is_ready: true,
        })
    }

    pub(super) fn is_tls(&self) -> bool {
        self.stream.is_tls()
    }

    pub(super) fn shutdown(&self) -> crate::Result<()> {
        Ok(self.stream.shutdown(Shutdown::Both)?)
    }

    #[inline]
    pub(super) async fn send<T>(&mut self, packet: T, initial: bool) -> crate::Result<()>
    where
        T: Encode + std::fmt::Debug,
    {
        if initial {
            self.seq_no = 0;
        }

        self.write(packet);
        self.flush().await
    }

    #[inline]
    pub(super) async fn flush(&mut self) -> crate::Result<()> {
        Ok(self.stream.flush().await?)
    }

    /// Write the packet to the buffered stream ( do not send to the server )
    pub(super) fn write<T>(&mut self, packet: T)
    where
        T: Encode,
    {
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

        LittleEndian::write_u32(&mut header, len as u32);

        // Take the last sequence number received, if any, and increment by 1
        // If there was no sequence number, we only increment if we split packets
        header[3] = self.seq_no;
        self.seq_no = self.seq_no.wrapping_add(1);
    }

    #[inline]
    pub(super) async fn receive(&mut self) -> crate::Result<&[u8]> {
        self.read().await?;

        Ok(self.packet())
    }

    pub(super) async fn read(&mut self) -> crate::Result<()> {
        self.packet_buf.clear();
        self.packet_len = 0;

        // Read the packet header which contains the length and the sequence number
        // https://dev.mysql.com/doc/dev/mysql-server/8.0.12/page_protocol_basic_packets.html
        // https://mariadb.com/kb/en/library/0-packet/#standard-packet
        let mut header = self.stream.peek(4_usize).await?;

        self.packet_len = header.get_uint::<LittleEndian>(3)? as usize;
        self.seq_no = header.get_u8()?.wrapping_add(1);

        self.stream.consume(4);

        // Read the packet body and copy it into our internal buf
        // We must have a separate buffer around the stream as we can't operate directly
        // on bytes returned from the stream. We have various kinds of payload manipulation
        // that must be handled before decoding.
        let payload = self.stream.peek(self.packet_len).await?;

        self.packet_buf.reserve(payload.len());
        self.packet_buf.extend_from_slice(payload);

        self.stream.consume(self.packet_len);

        // TODO: Implement packet compression
        // TODO: Implement packet joining

        Ok(())
    }

    /// Returns a reference to the most recently received packet data.
    /// A call to `read` invalidates this buffer.
    #[inline]
    pub(super) fn packet(&self) -> &[u8] {
        &self.packet_buf[..self.packet_len]
    }
}

impl MySqlStream {
    pub(crate) async fn maybe_receive_eof(&mut self) -> crate::Result<()> {
        if !self.capabilities.contains(Capabilities::DEPRECATE_EOF) {
            let _eof = EofPacket::read(self.receive().await?)?;
        }

        Ok(())
    }

    pub(crate) fn maybe_handle_eof(&mut self) -> crate::Result<Option<EofPacket>> {
        if !self.capabilities.contains(Capabilities::DEPRECATE_EOF) && self.packet()[0] == 0xFE {
            Ok(Some(EofPacket::read(self.packet())?))
        } else {
            Ok(None)
        }
    }

    pub(crate) fn handle_unexpected<T>(&mut self) -> crate::Result<T> {
        Err(protocol_err!("unexpected packet identifier 0x{:X?}", self.packet()[0]).into())
    }

    pub(crate) fn handle_err<T>(&mut self) -> crate::Result<T> {
        self.is_ready = true;
        Err(MySqlError(ErrPacket::read(self.packet(), self.capabilities)?).into())
    }

    pub(crate) fn handle_ok(&mut self) -> crate::Result<OkPacket> {
        self.is_ready = true;
        OkPacket::read(self.packet())
    }

    pub(crate) async fn wait_until_ready(&mut self) -> crate::Result<()> {
        if !self.is_ready {
            loop {
                let packet_id = self.receive().await?[0];
                match packet_id {
                    0xFE if self.packet().len() < 0xFF_FF_FF => {
                        // OK or EOF packet
                        self.is_ready = true;
                        break;
                    }

                    0xFF => {
                        // ERR packet
                        self.is_ready = true;
                        return self.handle_err();
                    }

                    _ => {
                        // Something else; skip
                    }
                }
            }
        }

        Ok(())
    }
}
