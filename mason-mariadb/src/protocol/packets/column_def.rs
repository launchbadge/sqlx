use super::super::{
    deserialize::{DeContext, Deserialize},
    types::{FieldDetailFlag, FieldType},
};
use bytes::Bytes;
use failure::Error;
use std::convert::TryFrom;

#[derive(Debug, Default)]
// ColumnDefPacket doesn't have a packet header because
// it's nested inside a result set packet
pub struct ColumnDefPacket {
    pub catalog: Bytes,
    pub schema: Bytes,
    pub table_alias: Bytes,
    pub table: Bytes,
    pub column_alias: Bytes,
    pub column: Bytes,
    pub length_of_fixed_fields: Option<usize>,
    pub char_set: u16,
    pub max_columns: u32,
    pub field_type: FieldType,
    pub field_details: FieldDetailFlag,
    pub decimals: u8,
}

impl Deserialize for ColumnDefPacket {
    fn deserialize(ctx: &mut DeContext) -> Result<Self, Error> {
        let decoder = &mut ctx.decoder;

        let catalog = decoder.decode_string_lenenc();
        let schema = decoder.decode_string_lenenc();
        let table_alias = decoder.decode_string_lenenc();
        let table = decoder.decode_string_lenenc();
        let column_alias = decoder.decode_string_lenenc();
        let column = decoder.decode_string_lenenc();
        let length_of_fixed_fields = decoder.decode_int_lenenc();
        let char_set = decoder.decode_int_2();
        let max_columns = decoder.decode_int_4();
        let field_type = FieldType::try_from(decoder.decode_int_1())?;
        let field_details = FieldDetailFlag::from_bits_truncate(decoder.decode_int_2());
        let decimals = decoder.decode_int_1();

        // Skip last two unused bytes
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
    use crate::{__bytes_builder, connection::Connection, protocol::decode::Decoder};
    use bytes::Bytes;
    use mason_core::ConnectOptions;

    #[runtime::test]
    async fn it_decodes_column_def_packet() -> Result<(), Error> {
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
            // string<lenenc> catalog (always 'def')
            1u8, 0u8, 0u8, b'a',
            // string<lenenc> schema
            1u8, 0u8, 0u8, b'b',
            // string<lenenc> table alias
            1u8, 0u8, 0u8, b'c',
            // string<lenenc> table
            1u8, 0u8, 0u8, b'd',
            // string<lenenc> column alias
            1u8, 0u8, 0u8, b'e',
            // string<lenenc> column
            1u8, 0u8, 0u8, b'f',
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

        let message = ColumnDefPacket::deserialize(&mut DeContext::new(&mut conn.context, &buf))?;

        assert_eq!(&message.catalog[..], b"a");
        assert_eq!(&message.schema[..], b"b");
        assert_eq!(&message.table_alias[..], b"c");
        assert_eq!(&message.table[..], b"d");
        assert_eq!(&message.column_alias[..], b"e");
        assert_eq!(&message.column[..], b"f");

        Ok(())
    }
}
