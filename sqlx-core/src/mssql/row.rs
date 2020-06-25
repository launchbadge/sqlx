use crate::error::Error;
use crate::mssql::protocol::row::Row as ProtocolRow;
use crate::mssql::{Mssql, MssqlValueRef};
use crate::row::{ColumnIndex, Row};

pub struct MssqlRow {
    pub(crate) row: ProtocolRow,
}

impl crate::row::private_row::Sealed for MssqlRow {}

impl Row for MssqlRow {
    type Database = Mssql;

    #[inline]
    fn len(&self) -> usize {
        self.row.values.len()
    }

    fn try_get_raw<I>(&self, index: I) -> Result<MssqlValueRef<'_>, Error>
    where
        I: ColumnIndex<Self>,
    {
        let index = index.index(self)?;
        let value = MssqlValueRef {
            data: self.row.values[index].as_ref(),
            type_info: self.row.column_types[index].clone(),
        };

        Ok(value)
    }
}
