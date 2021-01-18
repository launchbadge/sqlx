use crate::column::ColumnIndex;
use crate::error::Error;
use crate::postgres::message::DataRow;
use crate::postgres::statement::PgStatementMetadata;
use crate::postgres::value::PgValueFormat;
use crate::postgres::{PgColumn, PgValueRef, Postgres};
use crate::row::Row;
use std::fmt::{self, Debug};
use std::sync::Arc;

/// Implementation of [`Row`] for PostgreSQL.
pub struct PgRow {
    pub(crate) data: DataRow,
    pub(crate) format: PgValueFormat,
    pub(crate) metadata: Arc<PgStatementMetadata>,
}

impl crate::row::private_row::Sealed for PgRow {}

impl Debug for PgRow {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut map = f.debug_map();

        for (i, col) in self.columns().iter().enumerate() {
            if let Ok(val) = self.try_get::<bool, _>(i) {
                map.entry(&col.name, &val);
            } else if let Ok(val) = self.try_get::<i8, _>(i) {
                map.entry(&col.name, &val);
            } else if let Ok(val) = self.try_get::<i16, _>(i) {
                map.entry(&col.name, &val);
            } else if let Ok(val) = self.try_get::<i32, _>(i) {
                map.entry(&col.name, &val);
            } else if let Ok(val) = self.try_get::<i64, _>(i) {
                map.entry(&col.name, &val);
            } else if let Ok(val) = self.try_get::<f32, _>(i) {
                map.entry(&col.name, &val);
            } else if let Ok(val) = self.try_get::<f64, _>(i) {
                map.entry(&col.name, &val);
            } else if let Ok(val) = self.try_get::<&str, _>(i) {
                map.entry(&col.name, &val);
            } else {
                map.entry(&col.name, &"<unsupported in debug formatting>");
            }
        }

        map.finish()
    }
}

impl Row for PgRow {
    type Database = Postgres;

    fn columns(&self) -> &[PgColumn] {
        &self.metadata.columns
    }

    fn try_get_raw<I>(&self, index: I) -> Result<PgValueRef<'_>, Error>
    where
        I: ColumnIndex<Self>,
    {
        let index = index.index(self)?;
        let column = &self.metadata.columns[index];
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
        row.metadata
            .column_names
            .get(*self)
            .ok_or_else(|| Error::ColumnNotFound((*self).into()))
            .map(|v| *v)
    }
}

#[cfg(feature = "any")]
impl From<PgRow> for crate::any::AnyRow {
    #[inline]
    fn from(row: PgRow) -> Self {
        crate::any::AnyRow {
            columns: row
                .metadata
                .columns
                .iter()
                .map(|col| col.clone().into())
                .collect(),

            kind: crate::any::row::AnyRowKind::Postgres(row),
        }
    }
}
