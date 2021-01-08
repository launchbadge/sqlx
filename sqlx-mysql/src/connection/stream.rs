//! Reads and writes packets to and from the MySQL database server.
//!
//! The logic for serializing data structures into the packets is found
//! mostly in `protocol/`.
//!
//! Packets in MySQL are prefixed by 4 bytes.
//! 3 for length (in LE) and a sequence id.
//!
//! Packets may only be as large as the communicated size in the initial
//! `HandshakeResponse`. By default, SQLx configures its chunk size to 16M. Sending
//! a larger payload is simply sending completely "full" packets, one after the
//! other, with an increasing sequence id.
//!
//! In other words, when we sent data, we:
//!
//! -   Split the data into "packets" of size `2 ** 24 - 1` bytes.
//!
//! -   Prepend each packet with a **packet header**, consisting of the length of that packet,
//!     and the sequence number.
//!
//! https://dev.mysql.com/doc/internals/en/mysql-packet.html
//!
use bytes::{Buf, BufMut};
use sqlx_core::io::{Deserialize, Serialize};
use sqlx_core::{Error, Result, Runtime};

use crate::protocol::{Capabilities, ErrPacket};
use crate::{MySqlConnection, MySqlDatabaseError};

impl<Rt> MySqlConnection<Rt>
where
    Rt: Runtime,
{
    pub(super) fn write_packet<'ser, T>(&'ser mut self, packet: &T) -> Result<()>
    where
        T: Serialize<'ser, Capabilities>,
    {
        // the sequence-id is incremented with each packet and may
        // wrap around. it starts at 0 and is reset to 0 when a new command
        // begins in the Command Phase

        self.sequence_id = self.sequence_id.wrapping_add(1);

        // optimize for <16M packet sizes, in the case of >= 16M we would
        // swap out the write buffer for a fresh buffer and then split it into
        // 16M chunks separated by packet headers

        let buf = self.stream.buffer();
        let pos = buf.len();

        // leave room for the length of the packet header at the start
        buf.reserve(4);
        buf.extend_from_slice(&[0_u8; 3]);
        buf.push(self.sequence_id);

        // serialize the passed packet structure directly into the write buffer
        packet.serialize_with(buf, self.capabilities)?;

        let payload_len = buf.len() - pos - 4;

        // FIXME: handle split packets
        assert!(payload_len < 0xFF_FF_FF);

        // write back the length of the packet
        #[allow(clippy::cast_possible_truncation)]
        (&mut buf[pos..]).put_uint_le(payload_len as u64, 3);

        Ok(())
    }

    fn recv_packet<'de, T>(&'de mut self, len: usize) -> Result<T>
    where
        T: Deserialize<'de, Capabilities>,
    {
        // FIXME: handle split packets
        assert_ne!(len, 0xFF_FF_FF);

        // We store the sequence id here. To respond to a packet, it should use a
        // sequence id of n+1. It only "resets" at the start of a new command.
        self.sequence_id = self.stream.get(3, 1).get_u8();

        // tell the stream that we are done with the 4-byte header
        self.stream.consume(4);

        // and remove the remainder of the packet from the stream, the payload
        let payload = self.stream.take(len);

        if payload[0] == 0xff {
            // if the first byte of the payload is 0xFF and the payload is an ERR packet
            let err = ErrPacket::deserialize_with(payload, self.capabilities)?;
            return Err(Error::connect(MySqlDatabaseError(err)));
        }

        T::deserialize_with(payload, self.capabilities)
    }
}

macro_rules! read_packet {
    ($(@$blocking:ident)? $self:ident) => {{
        // reads at least 4 bytes from the IO stream into the read buffer
        read_packet!($(@$blocking)? @stream $self, 0, 4);

        // the first 3 bytes will be the payload length of the packet (in LE)
        // ALLOW: the max this len will be is 16M
        #[allow(clippy::cast_possible_truncation)]
        let payload_len: usize = $self.stream.get(0, 3).get_uint_le(3) as usize;

        // read <payload_len> bytes _after_ the 4 byte packet header
        // note that we have not yet told the stream we are done with any of
        // these bytes yet. if this next read invocation were to never return (eg., the
        // outer future was dropped), then the next time read_packet_async was called
        // it will re-read the parsed-above packet header. Note that we have NOT
        // mutated `self` _yet_. This is important.
        read_packet!($(@$blocking)? @stream $self, 4, payload_len);

        $self.recv_packet(payload_len)
    }};

    (@blocking @stream $self:ident, $offset:expr, $n:expr) => {
        $self.stream.read($offset, $n)?;
    };

    (@stream $self:ident, $offset:expr, $n:expr) => {
        $self.stream.read_async($offset, $n).await?;
    };
}

#[cfg(feature = "async")]
impl<Rt> MySqlConnection<Rt>
where
    Rt: sqlx_core::AsyncRuntime,
    <Rt as Runtime>::TcpStream: Unpin + futures_io::AsyncWrite + futures_io::AsyncRead,
{
    pub(super) async fn read_packet_async<'de, T>(&'de mut self) -> Result<T>
    where
        T: Deserialize<'de, Capabilities>,
    {
        read_packet!(self)
    }
}

#[cfg(feature = "blocking")]
impl<Rt> MySqlConnection<Rt>
where
    Rt: Runtime,
    <Rt as Runtime>::TcpStream: std::io::Write + std::io::Read,
{
    pub(super) fn read_packet<'de, T>(&'de mut self) -> Result<T>
    where
        T: Deserialize<'de, Capabilities>,
    {
        read_packet!(@blocking self)
    }
}
