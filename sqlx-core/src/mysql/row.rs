use std::collections::HashMap;
use std::sync::Arc;

use crate::decode::Decode;
use crate::mysql::protocol;
use crate::mysql::MySql;
use crate::row::{Row, RowIndex};
use crate::types::Type;

pub struct MySqlRow {
    pub(super) row: protocol::Row,
    pub(super) columns: Arc<HashMap<Box<str>, usize>>,
}

impl Row for MySqlRow {
    type Database = MySql;

    fn len(&self) -> usize {
        self.row.len()
    }

    fn get<T, I>(&self, index: I) -> T
    where
        Self::Database: Type<T>,
        I: RowIndex<Self>,
        T: Decode<Self::Database>,
    {
        index.try_get(self).unwrap()
    }
}

impl RowIndex<MySqlRow> for usize {
    fn try_get<T>(&self, row: &MySqlRow) -> crate::Result<T>
    where
        <MySqlRow as Row>::Database: Type<T>,
        T: Decode<<MySqlRow as Row>::Database>,
    {
        Ok(Decode::decode_nullable(row.row.get(*self))?)
    }
}

impl RowIndex<MySqlRow> for &'_ str {
    fn try_get<T>(&self, row: &MySqlRow) -> crate::Result<T>
    where
        <MySqlRow as Row>::Database: Type<T>,
        T: Decode<<MySqlRow as Row>::Database>,
    {
        let index = row
            .columns
            .get(*self)
            .ok_or_else(|| crate::Error::ColumnNotFound((*self).into()))?;

        let value = Decode::decode_nullable(row.row.get(*index))?;

        Ok(value)
    }
}

impl_from_row_for_row!(MySqlRow);
