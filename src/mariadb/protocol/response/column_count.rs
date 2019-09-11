use crate::mariadb::io::BufExt;
use byteorder::LittleEndian;
use std::io;

// The column packet is the first packet of a result set.
// Inside of it it contains the number of columns in the result set
// encoded as an int<lenenc>.
// https://mariadb.com/kb/en/library/resultset/#column-count-packet
#[derive(Debug)]
pub struct ColumnCountPacket {
    pub columns: u64,
}

impl ColumnCountPacket {
    pub(crate) fn decode(mut buf: &[u8]) -> io::Result<Self> {
        let columns = buf.get_uint_lenenc::<LittleEndian>()?.unwrap_or(0);

        Ok(Self { columns })
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::__bytes_builder;

    #[test]
    fn it_decodes_column_packet_0x_fb() -> io::Result<()> {
        #[rustfmt::skip]
        let buf = __bytes_builder!(
            // int<lenenc> tag code: Some(3 bytes)
            0xFD_u8,
            // value: 3 bytes
            0x01_u8, 0x01_u8, 0x01_u8
        );

        let message = ColumnCountPacket::decode(&buf)?;

        assert_eq!(message.columns, 0x010101);

        Ok(())
    }
}
