use bytes::Bytes;

use crate::error::Error;
use crate::io::BufExt;
use crate::mssql::protocol::col_meta_data::ColumnData;
use crate::mssql::MssqlTypeInfo;

#[derive(Debug)]
pub(crate) struct Row {
    // TODO: Column names?
    // FIXME: Columns Vec should be an Arc<_>
    pub(crate) column_types: Vec<MssqlTypeInfo>,
    pub(crate) values: Vec<Option<Bytes>>,
}

impl Row {
    pub(crate) fn get(
        buf: &mut Bytes,
        nullable: bool,
        columns: &[ColumnData],
    ) -> Result<Self, Error> {
        let mut values = Vec::with_capacity(columns.len());
        let mut column_types = Vec::with_capacity(columns.len());

        let nulls = if nullable {
            buf.get_bytes((columns.len() + 7) / 8)
        } else {
            Bytes::from_static(b"")
        };

        for (i, column) in columns.iter().enumerate() {
            column_types.push(MssqlTypeInfo(column.type_info.clone()));

            if !(column.type_info.is_null() || (nullable && (nulls[i / 8] & (1 << (i % 8))) != 0)) {
                values.push(column.type_info.get_value(buf));
            } else {
                values.push(None);
            }
        }

        Ok(Self {
            values,
            column_types,
        })
    }
}
