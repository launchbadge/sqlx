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
        if decoder.buf.len() < 4 {
            return Ok(None);
        }

        let length = decoder.decode_length()?;
        if decoder.buf.len() < (length + 4) as usize {
            return Ok(None);
        }

        let tag = decoder.buf[4];

        Ok(Some(match tag {
            0xFF => Message::ErrPacket(ErrPacket::deserialize(ctx)?),
            0x00 | 0xFE => Message::OkPacket(OkPacket::deserialize(ctx)?),
            _ => unimplemented!(),
        }))
    }
}
