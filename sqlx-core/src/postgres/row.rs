use std::collections::HashMap;
use std::sync::Arc;

use crate::postgres::protocol::{DataRow, TypeFormat};
use crate::postgres::type_info::SharedStr;
use crate::postgres::value::PgValue;
use crate::postgres::{PgTypeInfo, Postgres};
use crate::row::{ColumnIndex, Row};

// A statement has 0 or more columns being returned from the database
// For Postgres, each column has an OID and a format (binary or text)
// For simple (unprepared) queries, format will always be text
// For prepared queries, format will _almost_ always be binary
#[derive(Clone, Debug)]
pub(crate) struct Column {
    pub(crate) name: Option<SharedStr>,
    pub(crate) type_info: PgTypeInfo,
    pub(crate) format: TypeFormat,
    pub(crate) table_id: Option<u32>,
    pub(crate) column_id: i16,
}

// A statement description containing the column information used to
// properly decode data
#[derive(Default)]
pub(crate) struct Statement {
    // paramaters
    pub(crate) params: Box<[PgTypeInfo]>,

    // column name -> position
    pub(crate) names: HashMap<SharedStr, usize>,

    // all columns
    pub(crate) columns: Box<[Column]>,
}

pub struct PgRow<'c> {
    pub(super) data: DataRow<'c>,

    // shared reference to the statement this row is coming from
    // allows us to get the column information on demand
    pub(super) statement: Arc<Statement>,
}

impl crate::row::private_row::Sealed for PgRow<'_> {}

impl<'c> Row<'c> for PgRow<'c> {
    type Database = Postgres;

    #[inline]
    fn len(&self) -> usize {
        self.data.len()
    }

    #[doc(hidden)]
    fn try_get_raw<I>(&self, index: I) -> crate::Result<PgValue<'c>>
    where
        I: ColumnIndex<'c, Self>,
    {
        let index = index.index(self)?;
        let column = &self.statement.columns[index];
        let buffer = self.data.get(index);
        let value = match (column.format, buffer) {
            (_, None) => PgValue::null(),
            (TypeFormat::Binary, Some(buf)) => PgValue::bytes(column.type_info.clone(), buf),
            (TypeFormat::Text, Some(buf)) => PgValue::utf8(column.type_info.clone(), buf)?,
        };

        Ok(value)
    }
}

impl<'c> ColumnIndex<'c, PgRow<'c>> for usize {
    fn index(&self, row: &PgRow<'c>) -> crate::Result<usize> {
        let len = Row::len(row);

        if *self >= len {
            return Err(crate::Error::ColumnIndexOutOfBounds { len, index: *self });
        }

        Ok(*self)
    }
}

impl<'c> ColumnIndex<'c, PgRow<'c>> for str {
    fn index(&self, row: &PgRow<'c>) -> crate::Result<usize> {
        row.statement
            .names
            .get(self)
            .ok_or_else(|| crate::Error::ColumnNotFound((*self).into()))
            .map(|&index| index as usize)
    }
}
