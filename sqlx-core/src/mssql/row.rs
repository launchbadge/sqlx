use crate::error::Error;
use crate::mssql::protocol::row::Row as ProtocolRow;
use crate::mssql::{MsSql, MsSqlValueRef};
use crate::row::{ColumnIndex, Row};

pub struct MsSqlRow {
    pub(crate) row: ProtocolRow,
}

impl crate::row::private_row::Sealed for MsSqlRow {}

impl Row for MsSqlRow {
    type Database = MsSql;

    #[inline]
    fn len(&self) -> usize {
        self.row.values.len()
    }

    fn try_get_raw<I>(&self, index: I) -> Result<MsSqlValueRef<'_>, Error>
    where
        I: ColumnIndex<Self>,
    {
        let index = index.index(self)?;
        let value = MsSqlValueRef {
            data: self.row.values[index].as_ref(),
            type_info: self.row.column_types[index].clone(),
        };

        Ok(value)
    }
}
