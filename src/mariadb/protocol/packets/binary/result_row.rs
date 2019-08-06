use crate::mariadb::FieldType;
use bytes::Bytes;

#[derive(Debug, Default)]
pub struct ResultRow {
    pub columns: Vec<Option<Bytes>>,
}

impl crate::mariadb::Deserialize for ResultRow {
    fn deserialize(ctx: &mut crate::mariadb::DeContext) -> Result<Self, failure::Error> {
        let decoder = &mut ctx.decoder;

        let length = decoder.decode_length()?;
        let seq_no = decoder.decode_int_u8();

        let header = decoder.decode_int_u8();

        let bitmap = if let Some(columns) = ctx.columns {
            let size = (columns + 9) / 8;
            Ok(decoder.decode_byte_fix(size as usize))
        } else {
            Err(failure::err_msg(
                "Columns were not provided; cannot deserialize binary result row",
            ))
        }?;

        let row = match (&ctx.columns, &ctx.column_defs) {
            (Some(columns), Some(column_defs)) => {
                (0..*columns as usize)
                    .map(|index| {
                        if (1 << (index % 8)) & bitmap[index / 8] as usize == 0 {
                            None
                        } else {
                            match column_defs[index].field_type {
                                // Ordered by https://mariadb.com/kb/en/library/resultset-row/#binary-resultset-row
                                FieldType::MysqlTypeDouble => Some(decoder.decode_binary_double()),
                                FieldType::MysqlTypeLonglong => {
                                    Some(decoder.decode_binary_bigint())
                                }

                                // Is this MYSQL_TYPE_INTEGER?
                                FieldType::MysqlTypeLong => Some(decoder.decode_binary_int()),

                                // Is this MYSQL_TYPE_MEDIUMINTEGER?
                                FieldType::MysqlTypeInt24 => {
                                    Some(decoder.decode_binary_mediumint())
                                }

                                FieldType::MysqlTypeFloat => Some(decoder.decode_binary_float()),

                                // Is this MYSQL_TYPE_SMALLINT?
                                FieldType::MysqlTypeShort => Some(decoder.decode_binary_smallint()),

                                FieldType::MysqlTypeYear => Some(decoder.decode_binary_year()),
                                FieldType::MysqlTypeTiny => Some(decoder.decode_binary_tinyint()),
                                FieldType::MysqlTypeDate => Some(decoder.decode_binary_date()),
                                FieldType::MysqlTypeTimestamp => {
                                    Some(decoder.decode_binary_timestamp())
                                }
                                FieldType::MysqlTypeDatetime => {
                                    Some(decoder.decode_binary_datetime())
                                }
                                FieldType::MysqlTypeTime => Some(decoder.decode_binary_time()),
                                FieldType::MysqlTypeNewdecimal => {
                                    Some(decoder.decode_binary_decimal())
                                }

                                // This group of types are all encoded as byte<lenenc>
                                FieldType::MysqlTypeTinyBlob => Some(decoder.decode_byte_lenenc()),
                                FieldType::MysqlTypeMediumBlob => {
                                    Some(decoder.decode_byte_lenenc())
                                }
                                FieldType::MysqlTypeLongBlob => Some(decoder.decode_byte_lenenc()),
                                FieldType::MysqlTypeBlob => Some(decoder.decode_byte_lenenc()),
                                FieldType::MysqlTypeVarchar => Some(decoder.decode_byte_lenenc()),
                                FieldType::MysqlTypeVarString => Some(decoder.decode_byte_lenenc()),
                                FieldType::MysqlTypeString => Some(decoder.decode_byte_lenenc()),
                                FieldType::MysqlTypeGeometry => Some(decoder.decode_byte_lenenc()),

                                // The following did not have defined binary encoding, so I guessed.
                                // Perhaps you cannot get these types back from the server if you're using
                                // prepared statements? In that case we should error out here instead of
                                // proceeding to decode.
                                FieldType::MysqlTypeDecimal => {
                                    Some(decoder.decode_binary_decimal())
                                }
                                FieldType::MysqlTypeNull => panic!("Cannot decode MysqlTypeNull"),
                                FieldType::MysqlTypeNewdate => Some(decoder.decode_binary_date()),
                                FieldType::MysqlTypeBit => Some(decoder.decode_byte_fix(1)),
                                FieldType::MysqlTypeTimestamp2 => {
                                    Some(decoder.decode_binary_timestamp())
                                }
                                FieldType::MysqlTypeDatetime2 => {
                                    Some(decoder.decode_binary_datetime())
                                }
                                FieldType::MysqlTypeTime2 => Some(decoder.decode_binary_time()),
                                FieldType::MysqlTypeJson => Some(decoder.decode_byte_lenenc()),
                                FieldType::MysqlTypeEnum => Some(decoder.decode_byte_lenenc()),
                                FieldType::MysqlTypeSet => Some(decoder.decode_byte_lenenc()),
                            }
                        }
                    })
                    .collect::<Vec<Option<Bytes>>>()
            }
            _ => Vec::new(),
        };

        Ok(ResultRow::default())
    }
}
