use std::collections::HashMap;
use std::sync::Arc;

use crate::decode::Decode;
use crate::mysql::protocol;
use crate::mysql::MySql;
use crate::row::{Row, RowIndex};
use crate::types::HasSqlType;
use crate::cache::ColumnsData;
use crate::mysql::protocol::Type;

pub struct MySqlRow {
    pub(super) row: protocol::Row,
    pub(super) columns: Arc<ColumnsData<Type>>,
}

impl Row for MySqlRow {
    type Database = MySql;

    fn len(&self) -> usize {
        self.row.len()
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

impl RowIndex<MySqlRow> for usize {
    fn try_get<T>(&self, row: &MySqlRow) -> crate::Result<T>
    where
        <MySqlRow as Row>::Database: HasSqlType<T>,
        T: Decode<<MySqlRow as Row>::Database>,
    {
        row.columns.check_type::<MySql, T>(*self)?;
        Ok(Decode::decode_nullable(row.row.get(*self))?)
    }
}

impl RowIndex<MySqlRow> for &'_ str {
    fn try_get<T>(&self, row: &MySqlRow) -> crate::Result<T>
    where
        <MySqlRow as Row>::Database: HasSqlType<T>,
        T: Decode<<MySqlRow as Row>::Database>,
    {
        let index = row.columns.get_index(self)?;
        <usize as RowIndex<MySqlRow>>::try_get(&index, row)
    }
}

impl_from_row_for_row!(MySqlRow);
