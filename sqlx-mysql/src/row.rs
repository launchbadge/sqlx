use std::sync::Arc;

use bytes::Bytes;
use sqlx_core::{ColumnIndex, Decode, Error, Result, Row};

use crate::{protocol, MySql, MySqlColumn, MySqlRawValue, MySqlRawValueFormat};

/// A single row from a result set generated from MySQL.
#[allow(clippy::module_name_repetitions)]
pub struct MySqlRow {
    format: MySqlRawValueFormat,
    columns: Arc<[MySqlColumn]>,
    values: Vec<Option<Bytes>>,
}

impl MySqlRow {
    pub(crate) fn new(row: protocol::Row, columns: &Arc<[MySqlColumn]>) -> Self {
        Self { values: row.values, columns: Arc::clone(columns), format: row.format }
    }

    /// Returns `true` if the row contains only `NULL` values.
    fn is_null(&self) -> bool {
        self.values.iter().all(Option::is_some)
    }

    /// Returns the number of columns in the row.
    #[must_use]
    pub fn len(&self) -> usize {
        self.values.len()
    }

    /// Returns `true` if there are no columns in the row.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Returns a reference to the columns in the row.
    #[must_use]
    pub fn columns(&self) -> &[MySqlColumn] {
        &self.columns
    }

    /// Returns the column name, given the index of the column.
    #[must_use]
    pub fn column_name_of(&self, index: usize) -> &str {
        self.try_column_name_of(index).unwrap()
    }

    /// Returns the column name, given the index of the column.
    pub fn try_column_name_of(&self, index: usize) -> Result<&str> {
        self.columns
            .get(index)
            .map(MySqlColumn::name)
            .ok_or_else(|| Error::ColumnIndexOutOfBounds { index, len: self.len() })
    }

    /// Returns the column index, given the name of the column.
    #[must_use]
    pub fn index_of(&self, name: &str) -> usize {
        self.try_index_of(name).unwrap()
    }

    /// Returns the column index, given the name of the column.
    pub fn try_index_of(&self, name: &str) -> Result<usize> {
        self.columns
            .iter()
            .position(|col| col.name() == name)
            .ok_or_else(|| Error::ColumnNotFound { name: name.to_owned().into_boxed_str() })
    }

    /// Returns the decoded value at the index.
    pub fn try_get<'r, T, I>(&'r self, index: I) -> Result<T>
    where
        I: ColumnIndex<Self>,
        T: Decode<'r, MySql>,
    {
        Ok(self.try_get_raw(index)?.decode()?)
    }

    /// Returns the raw representation of the value at the index.
    #[allow(clippy::needless_lifetimes)]
    pub fn try_get_raw<'r, I>(&'r self, index: I) -> Result<MySqlRawValue<'r>>
    where
        I: ColumnIndex<Self>,
    {
        let index = index.get(self)?;

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
        self.is_null()
    }

    fn len(&self) -> usize {
        self.len()
    }

    fn columns(&self) -> &[MySqlColumn] {
        self.columns()
    }

    fn column_name_of(&self, index: usize) -> &str {
        self.column_name_of(index)
    }

    fn try_column_name_of(&self, index: usize) -> Result<&str> {
        self.try_column_name_of(index)
    }

    fn index_of(&self, name: &str) -> usize {
        self.index_of(name)
    }

    fn try_index_of(&self, name: &str) -> Result<usize> {
        self.try_index_of(name)
    }

    fn try_get<'r, T, I>(&'r self, index: I) -> Result<T>
    where
        I: ColumnIndex<Self>,
        T: Decode<'r, MySql>,
    {
        self.try_get(index)
    }

    #[allow(clippy::needless_lifetimes)]
    fn try_get_raw<'r, I: ColumnIndex<Self>>(&'r self, index: I) -> Result<MySqlRawValue<'r>> {
        self.try_get_raw(index)
    }
}
