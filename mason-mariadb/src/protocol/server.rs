// Reference: https://mariadb.com/kb/en/library/connection

use byteorder::{ByteOrder, LittleEndian};
use bytes::BytesMut;
use failure::Error;

use super::{
    decode::Decoder,
    deserialize::Deserialize,
    packets::{err::ErrPacket, initial::InitialHandshakePacket, ok::OkPacket},
};

#[derive(Debug)]
#[non_exhaustive]
pub enum Message {
    InitialHandshakePacket(InitialHandshakePacket),
    OkPacket(OkPacket),
    ErrPacket(ErrPacket),
}

impl Message {
    pub fn deserialize(buf: &mut BytesMut) -> Result<Option<Self>, Error> {
        if buf.len() < 4 {
            return Ok(None);
        }

        let length = LittleEndian::read_u24(&buf[0..]) as usize;
        if buf.len() < length + 4 {
            return Ok(None);
        }

        let buf = buf.split_to(length + 4).freeze();
        let _seq_no = [3];
        let tag = buf[4];

        Ok(Some(match tag {
            0xFF => Message::ErrPacket(ErrPacket::deserialize(&mut Decoder::new(&buf))?),
            0x00 | 0xFE => Message::OkPacket(OkPacket::deserialize(&mut Decoder::new(&buf))?),
            _ => unimplemented!(),
        }))
    }
}
