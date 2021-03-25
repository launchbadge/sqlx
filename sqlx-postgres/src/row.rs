use std::sync::Arc;

use bytes::Bytes;
use sqlx_core::{ColumnIndex, Result, Row, TypeDecode};

use crate::protocol::backend::DataRow;
use crate::{PgColumn, PgRawValue, Postgres};

/// A single result row from a query in PostgreSQL.
#[allow(clippy::module_name_repetitions)]
pub struct PgRow {
    values: Vec<Option<Bytes>>,
    columns: Arc<[PgColumn]>,
}

impl PgRow {
    pub(crate) fn new(data: DataRow, columns: &Option<Arc<[PgColumn]>>) -> Self {
        Self {
            values: data.values,
            columns: columns.as_ref().map(Arc::clone).unwrap_or_else(|| Arc::new([])),
        }
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
    pub fn columns(&self) -> &[PgColumn] {
        &self.columns
    }

    /// Returns the column at the index, if available.
    pub fn column<I: ColumnIndex<Self>>(&self, index: I) -> &PgColumn {
        Row::column(self, index)
    }

    /// Returns the column at the index, if available.
    pub fn try_column<I: ColumnIndex<Self>>(&self, index: I) -> Result<&PgColumn> {
        Ok(&self.columns[index.get(self)?])
    }

    /// Returns the decoded value at the index.
    pub fn get<'r, T, I>(&'r self, index: I) -> T
    where
        I: ColumnIndex<Self>,
        T: TypeDecode<'r, Postgres>,
    {
        Row::get(self, index)
    }

    /// Returns the decoded value at the index.
    pub fn try_get<'r, T, I>(&'r self, index: I) -> Result<T>
    where
        I: ColumnIndex<Self>,
        T: TypeDecode<'r, Postgres>,
    {
        Row::try_get(self, index)
    }

    /// Returns the raw representation of the value at the index.
    #[allow(clippy::needless_lifetimes)]
    pub fn get_raw<'r, I>(&'r self, index: I) -> PgRawValue<'r>
    where
        I: ColumnIndex<Self>,
    {
        Row::get_raw(self, index)
    }

    /// Returns the raw representation of the value at the index.
    #[allow(clippy::needless_lifetimes)]
    pub fn try_get_raw<'r, I>(&'r self, index: I) -> Result<PgRawValue<'r>>
    where
        I: ColumnIndex<Self>,
    {
        let index = index.get(self)?;

        let value = &self.values[index];
        let column = &self.columns[index];

        Ok(PgRawValue::new(value, column.format, column.type_info))
    }
}

impl Row for PgRow {
    type Database = Postgres;

    fn is_null(&self) -> bool {
        self.is_null()
    }

    fn len(&self) -> usize {
        self.len()
    }

    fn columns(&self) -> &[PgColumn] {
        self.columns()
    }

    fn try_column<I: ColumnIndex<Self>>(&self, index: I) -> Result<&PgColumn> {
        self.try_column(index)
    }

    fn column_name(&self, index: usize) -> Option<&str> {
        self.columns.get(index).map(PgColumn::name)
    }

    fn column_index(&self, name: &str) -> Option<usize> {
        self.columns.iter().position(|col| col.name() == name)
    }

    #[allow(clippy::needless_lifetimes)]
    fn try_get_raw<'r, I: ColumnIndex<Self>>(&'r self, index: I) -> Result<PgRawValue<'r>> {
        self.try_get_raw(index)
    }
}
