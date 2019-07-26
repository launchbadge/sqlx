use super::super::{
    decode::Decoder,
    deserialize::{DeContext, Deserialize},
    error_codes::ErrorCode,
    types::ServerStatusFlag,
};
use bytes::Bytes;
use failure::Error;
use std::convert::TryFrom;

#[derive(Default, Debug)]
pub struct EofPacket {
    pub length: u32,
    pub seq_no: u8,
    pub warning_count: u16,
    pub status: ServerStatusFlag,
}

impl Deserialize for EofPacket {
    fn deserialize(ctx: &mut DeContext) -> Result<Self, Error> {
        let decoder = &mut ctx.decoder;

        let length = decoder.decode_length()?;
        let seq_no = decoder.decode_int_1();

        let packet_header = decoder.decode_int_1();

        if packet_header != 0xFE {
            panic!("Packet header is not 0xFE for ErrPacket");
        }

        let warning_count = decoder.decode_int_2();
        let status = ServerStatusFlag::from_bits_truncate(decoder.decode_int_2());

        Ok(EofPacket { length, seq_no, warning_count, status })
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::{__bytes_builder, connection::Connection};
    use bytes::Bytes;
    use mason_core::ConnectOptions;

    #[runtime::test]
    async fn it_decodes_eof_packet() -> Result<(), Error> {
        let mut conn = Connection::establish(ConnectOptions {
            host: "127.0.0.1",
            port: 3306,
            user: Some("root"),
            database: None,
            password: None,
        })
        .await?;

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
        let _message = EofPacket::deserialize(&mut DeContext::new(&mut conn.context, &buf))?;

        Ok(())
    }
}
