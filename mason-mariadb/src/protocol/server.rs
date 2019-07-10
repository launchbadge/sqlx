// Reference: https://mariadb.com/kb/en/library/connection

use failure::Error;
use super::{
    decode::Decoder,
    deserialize::Deserialize,
    packets::{err::ErrPacket, initial::InitialHandshakePacket, ok::OkPacket},
};
use crate::connection::Connection;

#[derive(Debug)]
#[non_exhaustive]
pub enum Message {
    InitialHandshakePacket(InitialHandshakePacket),
    OkPacket(OkPacket),
    ErrPacket(ErrPacket),
}

impl Message {
    pub fn deserialize(conn: &mut Connection, decoder: &mut Decoder) -> Result<Option<Self>, Error> {
        if decoder.buf.len() < 4 {
            return Ok(None);
        }

        let length = decoder.decode_length()?;
        if decoder.buf.len() < (length + 4) as usize {
            return Ok(None);
        }

        let tag = decoder.buf[4];

        Ok(Some(match tag {
            0xFF => Message::ErrPacket(ErrPacket::deserialize(conn, decoder)?),
            0x00 | 0xFE => Message::OkPacket(OkPacket::deserialize(conn, decoder)?),
            _ => unimplemented!(),
        }))
    }
}
