use std::fmt::Debug;
use std::ops::{Deref, DerefMut};

use bytes::{Buf, BufMut};
use sqlx_core::io::{BufStream, Serialize, Stream};
use sqlx_core::net::Stream as NetStream;
use sqlx_core::{Result, Runtime};

use crate::protocol::{MaybeCommand, Packet, Quit};
use crate::MySqlDatabaseError;

/// Reads and writes packets to and from the MySQL database server.
///
/// The logic for serializing data structures into the packets is found
/// mostly in `protocol/`.
///
/// Packets in MySQL are prefixed by 4 bytes.
/// 3 for length (in LE) and a sequence id.
///
/// Packets may only be as large as the communicated size in the initial
/// `HandshakeResponse`. By default, SQLx configures its chunk size to 16M. Sending
/// a larger payload is simply sending completely "full" packets, one after the
/// other, with an increasing sequence id.
///
/// In other words, when we sent data, we:
///
/// -   Split the data into "packets" of size `2 ** 24 - 1` bytes.
///
/// -   Prepend each packet with a **packet header**, consisting of the length of that packet,
///     and the sequence number.
///
/// <https://dev.mysql.com/doc/internals/en/mysql-packet.html>
///
#[allow(clippy::module_name_repetitions)]
pub(crate) struct MySqlStream<Rt: Runtime> {
    stream: BufStream<Rt, NetStream<Rt>>,

    // the sequence-id is incremented with each packet and may wrap around. It starts at 0 and is
    // reset to 0 when a new command begins in the Command Phase.
    sequence_id: u8,
}

impl<Rt: Runtime> MySqlStream<Rt> {
    pub(crate) fn new(stream: NetStream<Rt>) -> Self {
        Self { stream: BufStream::with_capacity(stream, 4096, 1024), sequence_id: 0 }
    }

    pub(crate) fn write_packet<'ser, T>(&'ser mut self, packet: &T) -> Result<()>
    where
        T: Serialize<'ser> + Debug + MaybeCommand,
    {
        log::trace!("write > {:?}", packet);

        // the sequence-id is incremented with each packet and may
        // wrap around. it starts at 0 and is reset to 0 when a new command
        // begins in the Command Phase

        self.sequence_id = if T::is_command() { 0 } else { self.sequence_id.wrapping_add(1) };

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
        packet.serialize(buf)?;

        let payload_len = buf.len() - pos - 4;

        // FIXME: handle split packets
        assert!(payload_len < 0xFF_FF_FF);

        // write back the length of the packet
        #[allow(clippy::cast_possible_truncation)]
        (&mut buf[pos..]).put_uint_le(payload_len as u64, 3);

        Ok(())
    }

    // read and consumes a packet from the stream _buffer_
    // assumes there is a packet on the stream
    // is called by [read_packet_blocking] or [read_packet_async]
    fn read_packet(&mut self, len: usize) -> Result<Packet> {
        // We store the sequence id here. To respond to a packet, it should use a
        // sequence id of n+1. It only "resets" at the start of a new command.
        self.sequence_id = self.stream.get(3, 1).get_u8();

        // tell the stream that we are done with the 4-byte header
        self.stream.consume(4);

        // and remove the remainder of the packet from the stream, the payload
        let packet = Packet { bytes: self.stream.take(len) };

        if packet.bytes.len() != len {
            // BUG: something is very wrong somewhere if this branch is executed
            //      either in the SQLx MySQL driver or in the MySQL server
            return Err(MySqlDatabaseError::malformed_packet(&format!(
                "Received {} bytes for packet but expecting {} bytes",
                packet.bytes.len(),
                len
            ))
            .into());
        }

        Ok(packet)
    }
}

macro_rules! impl_read_packet {
    ($(@$blocking:ident)? $self:ident) => {{
        // reads at least 4 bytes from the IO stream into the read buffer
        impl_read_packet!($(@$blocking)? @stream $self, 0, 4);

        // the first 3 bytes will be the payload length of the packet (in LE)
        // ALLOW: the max this len will be is 16M
        #[allow(clippy::cast_possible_truncation)]
        let payload_len: usize = $self.stream.get(0, 3).get_uint_le(3) as usize;

        // read <payload_len> bytes _after_ the 4 byte packet header
        // note that we have not yet told the stream we are done with any of
        // these bytes yet. if this next read invocation were to never return (eg., the
        // outer future was dropped), then the next time read_packet was called
        // it will re-read the parsed-above packet header. Note that we have NOT
        // mutated `self` _yet_. This is important.
        impl_read_packet!($(@$blocking)? @stream $self, 4, payload_len);

        // FIXME: handle split packets
        assert_ne!(payload_len, 0xFF_FF_FF);

        $self.read_packet(payload_len)
    }};

    (@blocking @stream $self:ident, $offset:expr, $n:expr) => {
        $self.stream.read($offset, $n)?;
    };

    (@stream $self:ident, $offset:expr, $n:expr) => {
        $self.stream.read_async($offset, $n).await?;
    };
}

impl<Rt: Runtime> MySqlStream<Rt> {
    #[cfg(feature = "async")]
    pub(crate) async fn read_packet_async(&mut self) -> Result<Packet>
    where
        Rt: sqlx_core::Async,
    {
        impl_read_packet!(self)
    }

    #[cfg(feature = "blocking")]
    pub(crate) fn read_packet_blocking(&mut self) -> Result<Packet>
    where
        Rt: sqlx_core::blocking::Runtime,
    {
        impl_read_packet!(@blocking self)
    }
}

impl<Rt: Runtime> Deref for MySqlStream<Rt> {
    type Target = BufStream<Rt, NetStream<Rt>>;

    fn deref(&self) -> &Self::Target {
        &self.stream
    }
}

impl<Rt: Runtime> DerefMut for MySqlStream<Rt> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.stream
    }
}

macro_rules! read_packet {
    (@blocking $stream:expr) => {
        $stream.read_packet_blocking()?
    };

    ($stream:expr) => {
        $stream.read_packet_async().await?
    };
}

impl<Rt: Runtime> MySqlStream<Rt> {
    #[cfg(feature = "async")]
    pub(crate) async fn close_async(&mut self) -> Result<()>
    where
        Rt: sqlx_core::Async,
    {
        self.write_packet(&Quit)?;
        self.flush_async().await?;
        self.shutdown_async().await?;

        Ok(())
    }

    #[cfg(feature = "blocking")]
    pub(crate) fn close_blocking(&mut self) -> Result<()>
    where
        Rt: sqlx_core::blocking::Runtime,
    {
        self.write_packet(&Quit)?;
        self.flush()?;
        self.shutdown()?;

        Ok(())
    }
}
