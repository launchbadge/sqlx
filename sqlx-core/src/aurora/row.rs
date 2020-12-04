use crate::aurora::column::AuroraColumn;
use crate::aurora::statement::AuroraStatementMetadata;
use crate::aurora::value::AuroraValueRef;
use crate::aurora::Aurora;
use crate::column::ColumnIndex;
use crate::error::Error;
use crate::row::Row;

use rusoto_rds_data::Field;
use std::sync::Arc;

/// Implementation of [`Row`] for Aurora.
pub struct AuroraRow {
    pub(crate) fields: Vec<Field>,
    pub(crate) metadata: Arc<AuroraStatementMetadata>,
}

impl crate::row::private_row::Sealed for AuroraRow {}

impl Row for AuroraRow {
    type Database = Aurora;

    fn columns(&self) -> &[AuroraColumn] {
        &self.metadata.columns
    }

    fn try_get_raw<I>(&self, index: I) -> Result<AuroraValueRef<'_>, Error>
    where
        I: ColumnIndex<Self>,
    {
        let index = index.index(self)?;
        let field = &self.fields[index];
        let column = &self.metadata.columns[index];

        Ok(AuroraValueRef {
            field,
            type_info: column.type_info,
        })
    }
}

impl ColumnIndex<AuroraRow> for &'_ str {
    fn index(&self, row: &AuroraRow) -> Result<usize, Error> {
        row.metadata
            .column_names
            .get(*self)
            .ok_or_else(|| Error::ColumnNotFound((*self).into()))
            .map(|v| *v)
    }
}

#[cfg(feature = "any")]
impl From<AuroraRow> for crate::any::AnyRow {
    #[inline]
    fn from(row: AuroraRow) -> Self {
        crate::any::AnyRow {
            columns: row
                .metadata
                .columns
                .iter()
                .map(|col| col.clone().into())
                .collect(),

            kind: crate::any::row::AnyRowKind::Aurora(row),
        }
    }
}
