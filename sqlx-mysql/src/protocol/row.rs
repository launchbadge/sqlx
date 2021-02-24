use bytes::{Buf, Bytes};
use sqlx_core::io::Deserialize;
use sqlx_core::Result;

use crate::io::MySqlBufExt;
use crate::{MySqlColumn, MySqlRawValueFormat, MySqlTypeId};

#[derive(Debug)]
pub(crate) struct Row {
    pub(crate) format: MySqlRawValueFormat,
    pub(crate) values: Vec<Option<Bytes>>,
}

impl<'de> Deserialize<'de, (MySqlRawValueFormat, &'de [MySqlColumn])> for Row {
    fn deserialize_with(
        mut buf: Bytes,
        (format, columns): (MySqlRawValueFormat, &'de [MySqlColumn]),
    ) -> Result<Self> {
        let mut values = Vec::with_capacity(columns.len());

        if format == MySqlRawValueFormat::Text {
            for _ in columns {
                values.push(if buf.get(0).copied() == Some(0xfb) {
                    // in TEXT mode, NULL is transmitted as a single 0xFB byte
                    buf.advance(1);
                    None
                } else {
                    // otherwise, each value is a lenenc bytes

                    // the docs incorrectly state this is a string
                    // however, for binary values, these are the
                    // raw bytes, not hex encoded or something (like is
                    // done in postgres)

                    Some(buf.get_bytes_lenenc())
                });
            }
        } else {
            // https://dev.mysql.com/doc/dev/mysql-server/8.0.19/page_protocol_binary_resultset.html#sect_protocol_binary_resultset_row_value

            // [0x00] packer header
            let header = buf.get_u8();
            assert!(header == 0x00);

            // NULL bit map
            let null = buf.split_to((columns.len() + 9) / 8);

            // values for non-null columns
            // NULL columns are marked in the bitmap and are not in this list
            for (i, col) in columns.iter().enumerate() {
                // NOTE: the column index starts at the 3rd bit
                let null_i = i + 3;
                let is_null = null[null_i / 8] & (1 << (null_i % 8) as u8) != 0;

                if is_null {
                    values.push(None);
                    continue;
                }

                let size = match col.type_info().id() {
                    MySqlTypeId::TINYINT | MySqlTypeId::TINYINT_UNSIGNED => 1,
                    MySqlTypeId::SMALLINT | MySqlTypeId::SMALLINT_UNSIGNED => 2,
                    MySqlTypeId::BIGINT | MySqlTypeId::BIGINT_UNSIGNED => 8,

                    MySqlTypeId::MEDIUMINT
                    | MySqlTypeId::MEDIUMINT_UNSIGNED
                    | MySqlTypeId::INT
                    | MySqlTypeId::INT_UNSIGNED => 4,

                    MySqlTypeId::TEXT | MySqlTypeId::CHAR | MySqlTypeId::VARCHAR => {
                        buf.get_uint_lenenc()
                    }

                    id => {
                        // TODO: return a protocol error instead
                        unimplemented!("unsupported column type: {}", id.ty());
                    }
                };

                values.push(Some(buf.split_to(size as usize)));
            }
        }

        Ok(Self { format, values })
    }
}
