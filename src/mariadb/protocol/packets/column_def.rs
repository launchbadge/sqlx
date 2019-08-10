use crate::mariadb::{DeContext, Decode, FieldDetailFlag, FieldType};
use bytes::Bytes;
use failure::Error;
use std::convert::TryFrom;

#[derive(Debug, Default, Clone)]
// ColumnDefPacket doesn't have a packet header because
// it's nested inside a result set packet
pub struct ColumnDefPacket {
    pub catalog: Bytes,
    pub schema: Bytes,
    pub table_alias: Bytes,
    pub table: Bytes,
    pub column_alias: Bytes,
    pub column: Bytes,
    pub length_of_fixed_fields: Option<u64>,
    pub char_set: i16,
    pub max_columns: i32,
    pub field_type: FieldType,
    pub field_details: FieldDetailFlag,
    pub decimals: u8,
}

impl Decode for ColumnDefPacket {
    fn decode(ctx: &mut DeContext) -> Result<Self, Error> {
        let decoder = &mut ctx.decoder;
        let length = decoder.decode_length()?;
        let seq_no = decoder.decode_int_u8();

        // string<lenenc> catalog (always 'def')
        let catalog = decoder.decode_string_lenenc();
        // string<lenenc> schema
        let schema = decoder.decode_string_lenenc();
        // string<lenenc> table alias
        let table_alias = decoder.decode_string_lenenc();
        // string<lenenc> table
        let table = decoder.decode_string_lenenc();
        // string<lenenc> column alias
        let column_alias = decoder.decode_string_lenenc();
        // string<lenenc> column
        let column = decoder.decode_string_lenenc();
        // int<lenenc> length of fixed fields (=0xC)
        let length_of_fixed_fields = decoder.decode_int_lenenc_unsigned();
        // int<2> character set number
        let char_set = decoder.decode_int_i16();
        // int<4> max. column size
        let max_columns = decoder.decode_int_i32();
        // int<1> Field types
        let field_type = FieldType::try_from(decoder.decode_int_u8())?;
        // int<2> Field detail flag
        let field_details = FieldDetailFlag::from_bits_truncate(decoder.decode_int_u16());
        // int<1> decimals
        let decimals = decoder.decode_int_u8();
        // int<2> - unused -
        decoder.skip_bytes(2);

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
        __bytes_builder,
        mariadb::{ConnContext, Decoder},
        ConnectOptions,
    };
    use bytes::Bytes;

    #[test]
    fn it_decodes_column_def_packet() -> Result<(), Error> {
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

        let mut context = ConnContext::new();
        let mut ctx = DeContext::new(&mut context, buf);

        let message = ColumnDefPacket::decode(&mut ctx)?;

        assert_eq!(&message.catalog[..], b"a");
        assert_eq!(&message.schema[..], b"b");
        assert_eq!(&message.table_alias[..], b"c");
        assert_eq!(&message.table[..], b"d");
        assert_eq!(&message.column_alias[..], b"e");
        assert_eq!(&message.column[..], b"f");

        Ok(())
    }
}
