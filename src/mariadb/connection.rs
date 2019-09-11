use crate::{
    io::{Buf, BufMut, BufStream},
    mariadb::protocol::{ComPing, Encode},
};
use byteorder::{ByteOrder, LittleEndian};
use std::io;
use tokio::net::TcpStream;
use crate::mariadb::protocol::{OkPacket, ErrPacket, Capabilities};

pub struct Connection {
    stream: BufStream<TcpStream>,
    capabilities: Capabilities,
    next_seq_no: u8,
}

impl Connection {
    pub async fn ping(&mut self) -> crate::Result<()> {
        // Send the ping command and wait for (and drop) an OK packet

        self.start_sequence();
        self.write(ComPing);

        self.stream.flush().await?;

        let _ = self.receive_ok_or_err().await?;

        Ok(())
    }

    async fn receive(&mut self) -> crate::Result<&[u8]> {
        Ok(self
            .try_receive()
            .await?
            .ok_or(io::ErrorKind::UnexpectedEof)?)
    }

    async fn try_receive(&mut self) -> crate::Result<Option<&[u8]>> {
        // Read the packet header which contains the length and the sequence number
        // https://mariadb.com/kb/en/library/0-packet/#standard-packet
        let mut header = ret_if_none!(self.stream.peek(4).await?);
        let len = header.get_u24::<LittleEndian>()? as usize;
        self.next_seq_no = header.get_u8()? + 1;
        self.stream.consume(4);

        // Read the packet body and copy it into our internal buf
        // We must have a separate buffer around the stream as we can't operate directly
        // on bytes returend from the stream. We have compression, split, etc. to
        // unpack.
        let body = ret_if_none!(self.stream.peek(len).await?);
        self.rbuf.clear();
        self.rbuf.extend_from_slice(body);
        self.stream.consume(len);

        Ok(Some(&self.rbuf[..len]))
    }

    fn start_sequence(&mut self) {
        // At the start of a command sequence we reset our understanding
        // of [next_seq_no]. In a sequence our initial command must be 0, followed
        // by the server response that is 1, then our response to that response (if any),
        // would be 2
        self.next_seq_no = 0;
    }

    fn write<T: Encode>(&mut self, packet: T) {
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
        self.next_seq_no += 1;
    }

    // Decode an OK packet or bubble an ERR packet as an error
    // to terminate immediately
    async fn receive_ok_or_err(&mut self) -> crate::Result<OkPacket> {
        let mut buf = self.receive().await?;
        Ok(match buf[0] {
            0xfe | 0x00 => OkPacket::decode(buf, self.capabilities)?,

            0xff => {
                let err = ErrPacket::decode(buf)?;

                // TODO: Bubble as Error::Database
                panic!("received db err = {:?}", err);
            }

            id => {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    format!(
                        "unexpected packet identifier 0x{:X?} when expecting 0xFE (OK) or 0xFF (ERR)",
                        id
                    ),
                )
                    .into());
            }
        })
    }
}