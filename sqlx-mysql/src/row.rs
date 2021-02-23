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

    /// Returns the column name, given the ordinal (also known as index) of the column.
    #[must_use]
    pub fn column_name_of(&self, ordinal: usize) -> &str {
        self.try_column_name_of(ordinal).unwrap()
    }

    /// Returns the column name, given the ordinal (also known as index) of the column.
    pub fn try_column_name_of(&self, ordinal: usize) -> Result<&str> {
        self.columns
            .get(ordinal)
            .map(MySqlColumn::name)
            .ok_or_else(|| Error::ColumnIndexOutOfBounds { index: ordinal, len: self.len() })
    }

    /// Returns the column ordinal, given the name of the column.
    #[must_use]
    pub fn ordinal_of(&self, name: &str) -> usize {
        self.try_ordinal_of(name).unwrap()
    }

    /// Returns the column ordinal, given the name of the column.
    pub fn try_ordinal_of(&self, name: &str) -> Result<usize> {
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
        let ordinal = index.get(self)?;

        let value = self
            .values
            .get(ordinal)
            .ok_or_else(|| Error::ColumnIndexOutOfBounds { len: self.len(), index: ordinal })?;

        let column = &self.columns[ordinal];

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

    fn column_name_of(&self, ordinal: usize) -> &str {
        self.column_name_of(ordinal)
    }

    fn try_column_name_of(&self, ordinal: usize) -> Result<&str> {
        self.try_column_name_of(ordinal)
    }

    fn ordinal_of(&self, name: &str) -> usize {
        self.ordinal_of(name)
    }

    fn try_ordinal_of(&self, name: &str) -> Result<usize> {
        self.try_ordinal_of(name)
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
