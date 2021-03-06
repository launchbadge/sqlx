use sqlx_core::{ColumnIndex, Result, Row};

use crate::{PgColumn, PgRawValue, Postgres};

/// A single row from a result set generated from MySQL.
#[allow(clippy::module_name_repetitions)]
pub struct PgRow {}

impl Row for PgRow {
    type Database = Postgres;

    fn is_null(&self) -> bool {
        // self.is_null()
        todo!()
    }

    fn len(&self) -> usize {
        // self.len()
        todo!()
    }

    fn columns(&self) -> &[PgColumn] {
        // self.columns()
        todo!()
    }

    fn try_column<I: ColumnIndex<Self>>(&self, index: I) -> Result<&PgColumn> {
        // self.try_column(index)
        todo!()
    }

    fn column_name(&self, index: usize) -> Option<&str> {
        // self.columns.get(index).map(PgColumn::name)
        todo!()
    }

    fn column_index(&self, name: &str) -> Option<usize> {
        // self.columns.iter().position(|col| col.name() == name)
        todo!()
    }

    #[allow(clippy::needless_lifetimes)]
    fn try_get_raw<'r, I: ColumnIndex<Self>>(&'r self, index: I) -> Result<PgRawValue<'r>> {
        // self.try_get_raw(index)
        todo!()
    }
}
