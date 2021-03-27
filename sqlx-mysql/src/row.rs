use std::sync::Arc;

use bytes::Bytes;
use sqlx_core::{ColumnIndex, Result, Row, TypeDecode};

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
    pub fn is_null(&self) -> bool {
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

    /// Returns the column at the index, if available.
    pub fn column<I: ColumnIndex<Self>>(&self, index: I) -> &MySqlColumn {
        Row::column(self, index)
    }

    /// Returns the column at the index, if available.
    pub fn try_column<I: ColumnIndex<Self>>(&self, index: I) -> Result<&MySqlColumn> {
        Ok(&self.columns[index.get(self)?])
    }

    /// Returns the decoded value at the index.
    pub fn get<'r, T, I>(&'r self, index: I) -> T
    where
        I: ColumnIndex<Self>,
        T: TypeDecode<'r, MySql>,
    {
        Row::get(self, index)
    }

    /// Returns the decoded value at the index.
    pub fn try_get<'r, T, I>(&'r self, index: I) -> Result<T>
    where
        I: ColumnIndex<Self>,
        T: TypeDecode<'r, MySql>,
    {
        Row::try_get(self, index)
    }

    /// Returns the raw representation of the value at the index.
    #[allow(clippy::needless_lifetimes)]
    pub fn get_raw<'r, I>(&'r self, index: I) -> MySqlRawValue<'r>
    where
        I: ColumnIndex<Self>,
    {
        Row::get_raw(self, index)
    }

    /// Returns the raw representation of the value at the index.
    #[allow(clippy::needless_lifetimes)]
    pub fn try_get_raw<'r, I>(&'r self, index: I) -> Result<MySqlRawValue<'r>>
    where
        I: ColumnIndex<Self>,
    {
        let index = index.get(self)?;

        let value = &self.values[index];
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

    fn try_column<I: ColumnIndex<Self>>(&self, index: I) -> Result<&MySqlColumn> {
        self.try_column(index)
    }

    fn column_name(&self, index: usize) -> Option<&str> {
        self.columns.get(index).map(MySqlColumn::name)
    }

    fn column_index(&self, name: &str) -> Option<usize> {
        self.columns.iter().position(|col| col.name() == name)
    }

    #[allow(clippy::needless_lifetimes)]
    fn try_get_raw<'r, I: ColumnIndex<Self>>(&'r self, index: I) -> Result<MySqlRawValue<'r>> {
        self.try_get_raw(index)
    }
}
