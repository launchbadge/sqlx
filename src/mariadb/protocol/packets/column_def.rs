use crate::mariadb::{BufExt, Decode, FieldDetailFlag, FieldType, Capabilities};
use crate::io::Buf;
use std::io;
use byteorder::LittleEndian;

#[derive(Debug, Default, Clone)]
// ColumnDefPacket doesn't have a packet header because
// it's nested inside a result set packet
pub struct ColumnDefPacket<'a> {
    pub catalog: &'a str,
    pub schema: &'a str,
    pub table_alias: &'a str,
    pub table: &'a str,
    pub column_alias: &'a str,
    pub column: &'a str,
    pub length_of_fixed_fields: Option<u64>,
    pub char_set: u16,
    pub max_columns: i32,
    pub field_type: FieldType,
    pub field_details: FieldDetailFlag,
    pub decimals: u8,
}

impl<'a> Decode<'a> for ColumnDefPacket<'a> {
    fn decode(buf: &'a [u8], _: Capabilities) -> io::Result<Self> {
        // string<lenenc> catalog (always 'def')
        let catalog: &'a str = buf.get_str_lenenc::<LittleEndian>()?;
        // string<lenenc> schema
        let schema: &'a str = buf.get_str_lenenc::<LittleEndian>()?;
        // string<lenenc> table alias
        let table_alias: &'a str = buf.get_str_lenenc::<LittleEndian>()?;
        // string<lenenc> table
        let table: &'a str = buf.get_str_lenenc::<LittleEndian>()?;
        // string<lenenc> column alias
        let column_alias: &'a str = buf.get_str_lenenc::<LittleEndian>()?;
        // string<lenenc> column
        let column: &'a str = buf.get_str_lenenc::<LittleEndian>()?;
        // int<lenenc> length of fixed fields (=0xC)
        let length_of_fixed_fields = buf.get_uint_lenenc::<LittleEndian>()?;
        // int<2> character set number
        let char_set = buf.get_u16::<LittleEndian>()?;
        // int<4> max. column size
        let max_columns = buf.get_i32::<LittleEndian>()?;
        // int<1> Field types
        let field_type = FieldType(buf.get_u8()?);
        // int<2> Field detail flag
        let field_details = FieldDetailFlag::from_bits_truncate(buf.get_u16::<LittleEndian>()?);
        // int<1> decimals
        let decimals = buf.get_u8()?;
        // int<2> - unused -
        buf.advance(2);

        Ok(ColumnDefPacket {
            catalog,
            schema,
            table_alias,
            table,
            column_alias,
            column,
            length_of_fixed_fields,
            char_set,
            max_columns,
            field_type,
            field_details,
            decimals,
        })
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::{
        __bytes_builder};

    #[test]
    fn it_decodes_column_def_packet() -> io::Result<()> {
        #[rustfmt::skip]
        let buf = __bytes_builder!(
            // length
            1u8, 0u8, 0u8,
            // seq_no
            0u8,
            // string<lenenc> catalog (always 'def')
            1u8, b'a',
            // string<lenenc> schema
            1u8, b'b',
            // string<lenenc> table alias
            1u8, b'c',
            // string<lenenc> table
            1u8, b'd',
            // string<lenenc> column alias
            1u8, b'e',
            // string<lenenc> column
            1u8, b'f',
            // int<lenenc> length of fixed fields (=0xC)
            0xFC_u8, 1u8, 1u8,
            // int<2> character set number
            1u8, 1u8,
            // int<4> max. column size
            1u8, 1u8, 1u8, 1u8,
            // int<1> Field types
            1u8,
            // int<2> Field detail flag
            1u8, 0u8,
            // int<1> decimals
            1u8,
            // int<2> - unused -
            0u8, 0u8
        );

        let message = ColumnDefPacket::decode(&buf, Capabilities::CLIENT_PROTOCOL_41)?;

        assert_eq!(&message.catalog[..], b"a");
        assert_eq!(&message.schema[..], b"b");
        assert_eq!(&message.table_alias[..], b"c");
        assert_eq!(&message.table[..], b"d");
        assert_eq!(&message.column_alias[..], b"e");
        assert_eq!(&message.column[..], b"f");

        Ok(())
    }
}
