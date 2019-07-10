use std::convert::TryFrom;

use bytes::Bytes;
use failure::Error;

use super::super::{
    decode::Decoder,
    deserialize::Deserialize,
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
    fn deserialize(decoder: &mut Decoder) -> Result<Self, Error> {
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
