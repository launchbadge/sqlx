use crate::mariadb::{
    Decoder, DeContext, Deserialize, ErrorCode, ServerStatusFlag,
};
use bytes::Bytes;
use failure::Error;
use std::convert::TryFrom;

#[derive(Default, Debug)]
pub struct EofPacket {
    pub length: u32,
    pub seq_no: u8,
    pub warning_count: i16,
    pub status: ServerStatusFlag,
}

impl Deserialize for EofPacket {
    fn deserialize(ctx: &mut DeContext) -> Result<Self, Error> {
        let decoder = &mut ctx.decoder;

        let length = decoder.decode_length()?;
        let seq_no = decoder.decode_int_u8();

        let packet_header = decoder.decode_int_u8();

        if packet_header != 0xFE {
            panic!("Packet header is not 0xFE for ErrPacket");
        }

        let warning_count = decoder.decode_int_i16();
        let status = ServerStatusFlag::from_bits_truncate(decoder.decode_int_u16());

        Ok(EofPacket { length, seq_no, warning_count, status })
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::{__bytes_builder, ConnectOptions, mariadb::ConnContext};
    use bytes::Bytes;

    #[test]
    fn it_decodes_eof_packet() -> Result<(), Error> {
        #[rustfmt::skip]
        let buf = __bytes_builder!(
        // int<3> length
        1u8, 0u8, 0u8,
        // int<1> seq_no
        1u8,
        // int<1> 0xfe : EOF header
        0xFE_u8,
        // int<2> warning count
        0u8, 0u8,
        // int<2> server status
        1u8, 1u8
        );

        let buf = Bytes::from_static(b"\x01\0\0\x01\xFE\x00\x00\x01\x00");

        let mut context = ConnContext::new();
        let mut ctx = DeContext::new(&mut context, buf);

        let _message = EofPacket::deserialize(&mut ctx)?;

        Ok(())
    }
}
