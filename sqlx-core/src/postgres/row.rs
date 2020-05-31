use std::sync::Arc;

use hashbrown::HashMap;

use crate::error::Error;
use crate::ext::ustr::UStr;
use crate::postgres::message::DataRow;
use crate::postgres::value::PgValueFormat;
use crate::postgres::{PgTypeInfo, PgValueRef, Postgres};
use crate::row::{ColumnIndex, Row};

// Result column of a prepared statement
// See RowDescription/Field for more information
#[derive(Debug, Clone)]
pub(crate) struct PgColumn {
    pub(crate) name: UStr,
    pub(crate) type_info: PgTypeInfo,
    pub(crate) relation_id: Option<i32>,
    pub(crate) relation_attribute_no: Option<i16>,
}

/// Implementation of [`Row`] for PostgreSQL.
pub struct PgRow {
    pub(crate) data: DataRow,
    pub(crate) format: PgValueFormat,
    pub(crate) columns: Arc<Vec<PgColumn>>,
    pub(crate) column_names: Arc<HashMap<UStr, usize>>,
}

impl crate::row::private_row::Sealed for PgRow {}

impl Row for PgRow {
    type Database = Postgres;

    #[inline]
    fn len(&self) -> usize {
        self.data.len()
    }

    fn try_get_raw<I>(&self, index: I) -> Result<PgValueRef<'_>, Error>
    where
        I: ColumnIndex<Self>,
    {
        let index = index.index(self)?;
        let column = &self.columns[index];
        let value = self.data.get(index);

        Ok(PgValueRef {
            format: self.format,
            row: Some(&self.data.storage),
            type_info: column.type_info.clone(),
            value,
        })
    }
}

impl ColumnIndex<PgRow> for &'_ str {
    fn index(&self, row: &PgRow) -> Result<usize, Error> {
        row.column_names
            .get(*self)
            .ok_or_else(|| Error::ColumnNotFound((*self).into()))
            .map(|v| *v)
    }
}
