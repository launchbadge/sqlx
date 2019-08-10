use failure::Error;

use crate::mariadb::{DeContext, Decode};

// The column packet is the first packet of a result set.
// Inside of it it contains the number of columns in the result set
// encoded as an int<lenenc>.
#[derive(Default, Debug, Clone, Copy)]
pub struct ColumnPacket {
    pub length: u32,
    pub seq_no: u8,
    pub columns: Option<u64>,
}

impl Decode for ColumnPacket {
    fn decode(ctx: &mut DeContext) -> Result<Self, Error> {
        let decoder = &mut ctx.decoder;

        let length = decoder.decode_length()?;
        let seq_no = decoder.decode_int_u8();

        let mut columns = decoder.decode_int_lenenc_unsigned();

        // Treat 0 columns as None; this should never be a thing though
        if columns.is_some() && columns.unwrap() == 0 {
            columns = None;
        }

        Ok(ColumnPacket {
            length,
            seq_no,
            columns,
        })
    }
}

#[cfg(test)]
mod test {
    use bytes::Bytes;

    use super::*;
    use crate::{
        ConnectOptions, __bytes_builder,
        mariadb::{connection::ConnContext, protocol::decode::Decoder},
    };

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

        let message = ColumnPacket::decode(&mut ctx)?;

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

        let message = ColumnPacket::decode(&mut ctx)?;

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

        let message = ColumnPacket::decode(&mut ctx)?;

        assert_ne!(message.columns, Some(0x0100));

        Ok(())
    }
}
