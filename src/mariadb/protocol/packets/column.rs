use crate::mariadb::{BufExt, Decode, Capabilities};
use std::io;
use byteorder::LittleEndian;

// The column packet is the first packet of a result set.
// Inside of it it contains the number of columns in the result set
// encoded as an int<lenenc>.
#[derive(Default, Debug, Clone, Copy)]
pub struct ColumnPacket {
    pub columns: Option<u64>
}

impl Decode<'_> for ColumnPacket {
    fn decode(buf: &[u8], _: Capabilities) -> io::Result<Self> {
        let mut columns = buf.get_uint_lenenc::<LittleEndian>()?;

        // Treat 0 columns as None; this should never be a thing though
        if columns.is_some() && columns.unwrap() == 0 {
            columns = None;
        }

        Ok(ColumnPacket {
            columns,
        })
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::{
        __bytes_builder};

    #[test]
    fn it_decodes_column_packet_0x_fb() -> io::Result<()> {
        #[rustfmt::skip]
        let buf = __bytes_builder!(
        // int<3> length
        0u8, 0u8, 0u8,
        // int<1> seq_no
        0u8,
        // int<lenenc> tag code: None
        0xFB_u8
        );

        let message = ColumnPacket::decode(&buf, Capabilities::CLIENT_PROTOCOL_41)?;

        assert_eq!(message.columns, None);

        Ok(())
    }

    #[test]
    fn it_decodes_column_packet_0x_fd() -> io::Result<()> {
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

        let message = ColumnPacket::decode(&buf, Capabilities::CLIENT_PROTOCOL_41)?;

        assert_eq!(message.columns, Some(0x010101));

        Ok(())
    }

    #[test]
    fn it_fails_to_decode_column_packet_0x_fc() -> io::Result<()> {
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

        let message = ColumnPacket::decode(&buf, Capabilities::CLIENT_PROTOCOL_41)?;

        assert_ne!(message.columns, Some(0x0100));

        Ok(())
    }
}
