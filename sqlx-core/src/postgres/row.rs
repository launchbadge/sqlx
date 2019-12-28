use std::collections::HashMap;
use std::sync::Arc;

use crate::decode::Decode;
use crate::postgres::protocol::DataRow;
use crate::postgres::Postgres;
use crate::row::{Row, RowIndex};
use crate::types::HasSqlType;

pub struct PgRow {
    pub(super) data: DataRow,
    pub(super) columns: Arc<HashMap<Box<str>, usize>>,
}

impl Row for PgRow {
    type Database = Postgres;

    fn len(&self) -> usize {
        self.data.len()
    }

    fn get<T, I>(&self, index: I) -> T
    where
        Self::Database: HasSqlType<T>,
        I: RowIndex<Self>,
        T: Decode<Self::Database>,
    {
        index.try_get(self).unwrap()
    }
}

impl RowIndex<PgRow> for usize {
    fn try_get<T>(&self, row: &PgRow) -> crate::Result<T>
    where
        <PgRow as Row>::Database: HasSqlType<T>,
        T: Decode<<PgRow as Row>::Database>,
    {
        Ok(Decode::decode_nullable(row.data.get(*self))?)
    }
}

impl RowIndex<PgRow> for &'_ str {
    fn try_get<T>(&self, row: &PgRow) -> crate::Result<T>
    where
        <PgRow as Row>::Database: HasSqlType<T>,
        T: Decode<<PgRow as Row>::Database>,
    {
        let index = row
            .columns
            .get(*self)
            .ok_or_else(|| crate::Error::ColumnNotFound((*self).into()))?;
        let value = Decode::decode_nullable(row.data.get(*index))?;

        Ok(value)
    }
}

impl_from_row_for_row!(PgRow);
