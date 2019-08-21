use crate::mariadb::FieldType;
use bytes::Bytes;

#[derive(Debug, Default)]
pub struct ResultRow {
    pub length: u32,
    pub seq_no: u8,
    pub columns: Vec<Option<Bytes>>,
}

impl crate::mariadb::Decode for ResultRow {
    fn decode(ctx: &mut crate::mariadb::DeContext) -> Result<Self, failure::Error> {
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

        let columns = match (&ctx.columns, &ctx.column_defs) {
            (Some(columns), Some(column_defs)) => {
                (0..*columns as usize)
                    .map(|index| {
                        if (1 << (index % 8)) & bitmap[index / 8] as usize == 1 {
                            None
                        } else {
                            match column_defs[index].field_type {
                                // Ordered by https://mariadb.com/kb/en/library/resultset-row/#binary-resultset-row
                                FieldType::MYSQL_TYPE_DOUBLE => Some(decoder.decode_binary_double()),
                                FieldType::MYSQL_TYPE_LONGLONG => {
                                    Some(decoder.decode_binary_bigint())
                                }

                                // Is this MYSQL_TYPE_INTEGER?
                                FieldType::MYSQL_TYPE_LONG => Some(decoder.decode_binary_int()),

                                // Is this MYSQL_TYPE_MEDIUMINTEGER?
                                FieldType::MYSQL_TYPE_INT24 => {
                                    Some(decoder.decode_binary_mediumint())
                                }

                                FieldType::MYSQL_TYPE_FLOAT => Some(decoder.decode_binary_float()),

                                // Is this MYSQL_TYPE_SMALLINT?
                                FieldType::MYSQL_TYPE_SHORT => Some(decoder.decode_binary_smallint()),

                                FieldType::MYSQL_TYPE_YEAR => Some(decoder.decode_binary_year()),
                                FieldType::MYSQL_TYPE_TINY => Some(decoder.decode_binary_tinyint()),
                                FieldType::MYSQL_TYPE_DATE => Some(decoder.decode_binary_date()),
                                FieldType::MYSQL_TYPE_TIMESTAMP => {
                                    Some(decoder.decode_binary_timestamp())
                                }
                                FieldType::MYSQL_TYPE_DATETIME => {
                                    Some(decoder.decode_binary_datetime())
                                }
                                FieldType::MYSQL_TYPE_TIME => Some(decoder.decode_binary_time()),
                                FieldType::MYSQL_TYPE_NEWDECIMAL => {
                                    Some(decoder.decode_binary_decimal())
                                }

                                // This group of types are all encoded as byte<lenenc>
                                FieldType::MYSQL_TYPE_TINY_BLOB => Some(decoder.decode_byte_lenenc()),
                                FieldType::MYSQL_TYPE_MEDIUM_BLOB => {
                                    Some(decoder.decode_byte_lenenc())
                                }
                                FieldType::MYSQL_TYPE_LONG_BLOB => Some(decoder.decode_byte_lenenc()),
                                FieldType::MYSQL_TYPE_BLOB => Some(decoder.decode_byte_lenenc()),
                                FieldType::MYSQL_TYPE_VARCHAR => Some(decoder.decode_byte_lenenc()),
                                FieldType::MYSQL_TYPE_VAR_STRING => Some(decoder.decode_byte_lenenc()),
                                FieldType::MYSQL_TYPE_STRING => Some(decoder.decode_byte_lenenc()),
                                FieldType::MYSQL_TYPE_GEOMETRY => Some(decoder.decode_byte_lenenc()),

                                // The following did not have defined binary encoding, so I guessed.
                                // Perhaps you cannot get these types back from the server if you're using
                                // prepared statements? In that case we should error out here instead of
                                // proceeding to decode.
                                FieldType::MYSQL_TYPE_DECIMAL => {
                                    Some(decoder.decode_binary_decimal())
                                }
                                FieldType::MYSQL_TYPE_NULL => panic!("Cannot decode MysqlTypeNull"),
                                FieldType::MYSQL_TYPE_NEWDATE => Some(decoder.decode_binary_date()),
                                FieldType::MYSQL_TYPE_BIT => Some(decoder.decode_byte_fix(1)),
                                FieldType::MYSQL_TYPE_TIMESTAMP2 => {
                                    Some(decoder.decode_binary_timestamp())
                                }
                                FieldType::MYSQL_TYPE_DATETIME2 => {
                                    Some(decoder.decode_binary_datetime())
                                }
                                FieldType::MYSQL_TYPE_TIME2 => Some(decoder.decode_binary_time()),
                                FieldType::MYSQL_TYPE_JSON => Some(decoder.decode_byte_lenenc()),
                                FieldType::MYSQL_TYPE_ENUM => Some(decoder.decode_byte_lenenc()),
                                FieldType::MYSQL_TYPE_SET => Some(decoder.decode_byte_lenenc()),
                                _ => panic!("Unrecognized FieldType received from MaraiDB"),
                            }
                        }
                    })
                    .collect::<Vec<Option<Bytes>>>()
            }
            _ => Vec::new(),
        };

        Ok(ResultRow {
            length,
            seq_no,
            columns,
        })
    }
}
