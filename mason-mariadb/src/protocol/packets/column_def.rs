use std::convert::TryFrom;
use bytes::Bytes;
use failure::Error;
use super::super::{
    deserialize::{Deserialize, DeContext},
    types::{FieldDetailFlag, FieldType},
};

#[derive(Debug, Default)]
pub struct ColumnDefPacket {
    pub length: u32,
    pub seq_no: u8,
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
        let length = decoder.decode_length()?;
        let seq_no = decoder.decode_int_1();

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
            length,
            seq_no,
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
    use bytes::Bytes;
    use super::*;
    use mason_core::ConnectOptions;

    #[runtime::test]
    async fn it_decodes_column_def_packet() -> Result<(), Error> {
        let mut conn = Connection::establish(ConnectOptions {
            host: "127.0.0.1",
            port: 3306,
            user: Some("root"),
            database: None,
            password: None,
        }).await?;

        let buf = Bytes::from(b"\
        \0\0\0\
        \x01\
        \x01\0\0a\
        \x01\0\0b\
        \x01\0\0c\
        \x01\0\0d\
        \x01\0\0e\
        \x01\0\0f\
        \xfc\x01\x01\
        \x01\x01\
        \x01\x01\x01\x01\
        \x00\
        \x00\x00\
        \x01\
        \0\0
        ".to_vec());
        let message = ColumnDefPacket::deserialize(&mut conn, &mut Decoder::new(&buf))?;

        assert_eq!(&message.catalog[..], b"a");
        assert_eq!(&message.schema[..], b"b");
        assert_eq!(&message.table_alias[..], b"c");
        assert_eq!(&message.table[..], b"d");
        assert_eq!(&message.column_alias[..], b"e");
        assert_eq!(&message.column[..], b"f");

        Ok(())
    }
}
