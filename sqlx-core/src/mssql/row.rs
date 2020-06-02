use crate::error::Error;
use crate::mssql::{MsSql, MsSqlValueRef};
use crate::row::{ColumnIndex, Row};

pub struct MsSqlRow {}

impl crate::row::private_row::Sealed for MsSqlRow {}

impl Row for MsSqlRow {
    type Database = MsSql;

    #[inline]
    fn len(&self) -> usize {
        unimplemented!()
    }

    fn try_get_raw<I>(&self, index: I) -> Result<MsSqlValueRef<'_>, Error>
    where
        I: ColumnIndex<Self>,
    {
        unimplemented!()
    }
}
