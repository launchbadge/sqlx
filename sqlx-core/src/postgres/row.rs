use std::fmt::{self, Debug, Formatter};
use std::sync::Arc;

use crate::database::HasRawValue;
use crate::error::Error;
use crate::postgres::connection::describe::Statement;
use crate::postgres::message::DataRow;
use crate::postgres::value::PgValueFormat;
use crate::postgres::{PgRawValue, Postgres};
use crate::row::{ColumnIndex, Row};

// TODO: Do _not_ derive(Debug)

/// Implementation of [`Row`] for PostgreSQL.
#[derive(Debug)]
pub struct PgRow {
    pub(crate) data: DataRow,
    pub(crate) format: PgValueFormat,
    pub(crate) statement: Arc<Statement>,
}

impl Row for PgRow {
    type Database = Postgres;

    #[inline]
    fn len(&self) -> usize {
        self.data.len()
    }

    fn try_get_raw<I>(&self, index: I) -> Result<<Self::Database as HasRawValue>::RawValue, Error>
    where
        I: ColumnIndex<Self>,
    {
        let index = index.index(self)?;
        let value = self.data.get(index);

        Ok(PgRawValue {
            format: self.format,
            value,
        })
    }
}

impl crate::row::private_row::Sealed for PgRow {}

impl ColumnIndex<PgRow> for &'_ str {
    fn index(&self, row: &PgRow) -> Result<usize, Error> {
        row.statement
            .column_names
            .get(*self)
            .ok_or_else(|| Error::ColumnNotFound((*self).into()))
            .map(|v| *v)
    }
}
