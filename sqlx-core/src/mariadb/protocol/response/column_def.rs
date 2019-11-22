use crate::{
    io::Buf,
    mariadb::{
        io::BufExt,
        protocol::{FieldDetailFlag, FieldType},
    },
};
use byteorder::LittleEndian;
use std::io;

#[derive(Debug)]
// ColumnDefinitionPacket doesn't have a packet header because
// it's nested inside a result set packet
pub struct ColumnDefinitionPacket {
    pub schema: Option<String>,
    pub table_alias: Option<String>,
    pub table: Option<String>,
    pub column_alias: Option<String>,
    pub column: Option<String>,
    pub char_set: u16,
    pub max_columns: i32,
    pub field_type: FieldType,
    pub field_details: FieldDetailFlag,
    pub decimals: u8,
}

impl ColumnDefinitionPacket {
    pub(crate) fn decode(mut buf: &[u8]) -> io::Result<Self> {
        // string<lenenc> catalog (always 'def')
        let _catalog = buf.get_str_lenenc::<LittleEndian>()?;
        // TODO: Assert that this is always DEF

        // string<lenenc> schema
        let schema = buf.get_str_lenenc::<LittleEndian>()?.map(ToOwned::to_owned);
        // string<lenenc> table alias
        let table_alias = buf.get_str_lenenc::<LittleEndian>()?.map(ToOwned::to_owned);
        // string<lenenc> table
        let table = buf.get_str_lenenc::<LittleEndian>()?.map(ToOwned::to_owned);
        // string<lenenc> column alias
        let column_alias = buf.get_str_lenenc::<LittleEndian>()?.map(ToOwned::to_owned);
        // string<lenenc> column
        let column = buf.get_str_lenenc::<LittleEndian>()?.map(ToOwned::to_owned);

        // int<lenenc> length of fixed fields (=0xC)
        let _length_of_fixed_fields = buf.get_uint_lenenc::<LittleEndian>()?;
        // TODO: Assert that this is always 0xC

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

        Ok(Self {
            schema,
            table_alias,
            table,
            column_alias,
            column,
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
    use crate::__bytes_builder;

    #[test]
    fn it_decodes_column_def_packet() -> io::Result<()> {
        #[rustfmt::skip]
        let buf = __bytes_builder!(
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

        let message = ColumnDefinitionPacket::decode(&buf)?;

        assert_eq!(message.schema, Some("b".into()));
        assert_eq!(message.table_alias, Some("c".into()));
        assert_eq!(message.table, Some("d".into()));
        assert_eq!(message.column_alias, Some("e".into()));
        assert_eq!(message.column, Some("f".into()));

        Ok(())
    }
}
