use bytes::Bytes;
use sqlx_core::{Decode, Error, Row};

use crate::{protocol, MySql, MySqlColumn, MySqlRawValue, MySqlRawValueFormat};

#[allow(clippy::module_name_repetitions)]
pub struct MySqlRow {
    format: MySqlRawValueFormat,
    columns: Vec<MySqlColumn>,
    values: Vec<Option<Bytes>>,
}

impl MySqlRow {
    // FIXME: Use Arc or some other way of sharing columns between rows
    pub(crate) fn new(row: protocol::Row, columns: &Vec<MySqlColumn>) -> Self {
        Self { values: row.values, columns: columns.clone(), format: row.format }
    }

    #[must_use]
    pub fn len(&self) -> usize {
        self.values.len()
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub fn try_get<'r, T>(&'r self, index: usize) -> sqlx_core::Result<T>
    where
        T: Decode<'r, MySql>,
    {
        Ok(self.try_get_raw(index)?.decode()?)
    }

    // noinspection RsNeedlessLifetimes
    pub fn try_get_raw<'r>(&'r self, index: usize) -> sqlx_core::Result<MySqlRawValue<'r>> {
        let value = self
            .values
            .get(index)
            .ok_or_else(|| Error::ColumnIndexOutOfBounds { len: self.len(), index })?;

        let column = &self.columns[index];

        Ok(MySqlRawValue::new(value, self.format, column.type_info()))
    }
}

impl Row for MySqlRow {
    type Database = MySql;

    fn is_null(&self) -> bool {
        todo!()
    }

    fn len(&self) -> usize {
        self.len()
    }

    fn columns(&self) -> &[MySqlColumn] {
        todo!()
    }

    fn column_name_of(&self, ordinal: usize) -> &str {
        todo!()
    }

    fn try_column_name_of(&self, ordinal: usize) -> sqlx_core::Result<&str> {
        todo!()
    }

    fn ordinal_of(&self, name: &str) -> usize {
        todo!()
    }

    fn try_ordinal_of(&self, name: &str) -> sqlx_core::Result<usize> {
        todo!()
    }

    fn try_get<'r, T>(&'r self, index: usize) -> sqlx_core::Result<T>
    where
        T: Decode<'r, MySql>,
    {
        self.try_get(index)
    }

    // noinspection RsNeedlessLifetimes
    fn try_get_raw<'r>(&'r self, index: usize) -> sqlx_core::Result<MySqlRawValue<'r>> {
        self.try_get_raw(index)
    }
}
