// Reference: https://mariadb.com/kb/en/library/connection

use failure::Error;

use super::{
    deserialize::{DeContext, Deserialize},
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
    pub fn deserialize(ctx: &mut DeContext) -> Result<Option<Self>, Error> {
        let decoder = &mut ctx.decoder;
        let _packet_header = match decoder.peek_packet_header() {
            Ok(v) => v,
            Err(_) => return Ok(None),
        };

        let tag = match decoder.peek_tag() {
            Some(v) => v,
            None => return Ok(None),
        };

        Ok(Some(match tag {
            0xFF => Message::ErrPacket(ErrPacket::deserialize(ctx)?),
            0x00 | 0xFE => Message::OkPacket(OkPacket::deserialize(ctx)?),
            _ => unimplemented!(),
        }))
    }
}
