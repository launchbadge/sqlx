use failure::Error;

use crate::mariadb::{DeContext, Deserialize};

// The column packet is the first packet of a result set.
// Inside of it it contains the number of columns in the result set
// encoded as an int<lenenc>.
#[derive(Default, Debug, Clone, Copy)]
pub struct ColumnPacket {
    pub length: u32,
    pub seq_no: u8,
    pub columns: Option<u64>,
}

impl Deserialize for ColumnPacket {
    fn deserialize(ctx: &mut DeContext) -> Result<Self, Error> {
        let decoder = &mut ctx.decoder;

        let length = decoder.decode_length()?;
        let seq_no = decoder.decode_int_u8();

        let columns = decoder.decode_int_lenenc_unsigned();

        Ok(ColumnPacket { length, seq_no, columns })
    }
}

#[cfg(test)]
mod test {
    use bytes::Bytes;

    use crate::ConnectOptions;
    use crate::{__bytes_builder, mariadb::connection::ConnContext, mariadb::protocol::decode::Decoder};
    use super::*;

    #[test]
    fn it_decodes_column_packet_0x_fb() -> Result<(), Error> {
        #[rustfmt::skip]
        let buf = __bytes_builder!(
        // int<3> length
        0u8, 0u8, 0u8,
        // int<1> seq_no
        0u8,
        // int<lenenc> tag code: None
        0xFB_u8
        );

        let mut context = ConnContext::new();
        let mut ctx = DeContext::new(&mut context, buf);

        let message = ColumnPacket::deserialize(&mut ctx)?;

        assert_eq!(message.columns, None);

        Ok(())
    }

    #[test]
    fn it_decodes_column_packet_0x_fd() -> Result<(), Error> {
        #[rustfmt::skip]
        let buf = __bytes_builder!(
        // int<3> length
        0u8, 0u8, 0u8,
        // int<1> seq_no
        0u8,
        // int<lenenc> tag code: Some(3 bytes)
        0xFD_u8,
        // value: 3 bytes
        0x01_u8, 0x01_u8, 0x01_u8
        );

        let mut context = ConnContext::new();
        let mut ctx = DeContext::new(&mut context, buf);

        let message = ColumnPacket::deserialize(&mut ctx)?;

        assert_eq!(message.columns, Some(0x010101));

        Ok(())
    }

    #[test]
    fn it_fails_to_decode_column_packet_0x_fc() -> Result<(), Error> {
        #[rustfmt::skip]
        let buf = __bytes_builder!(
        // int<3> length
        0u8, 0u8, 0u8,
        // int<1> seq_no
        0u8,
        // int<lenenc> tag code: Some(3 bytes)
        0xFC_u8,
        // value: 2 bytes
        0x01_u8, 0x01_u8
        );

        let mut context = ConnContext::new();
        let mut ctx = DeContext::new(&mut context, buf);

        let message = ColumnPacket::deserialize(&mut ctx)?;

        assert_ne!(message.columns, Some(0x0100));

        Ok(())
    }
}
