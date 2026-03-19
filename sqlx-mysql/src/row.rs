use std::sync::Arc;
pub(crate) use sqlx_core::row::*;

use crate::column::ColumnIndex;
use crate::error::Error;
use crate::ext::ustr::UStr;
use crate::HashMap;
use crate::{protocol, MySql, MySqlColumn, MySqlValueFormat, MySqlValueRef};

/// Implementation of [`Row`] for MySQL.
pub struct MySqlRow {
    pub(crate) row: protocol::Row,
    pub(crate) format: MySqlValueFormat,
    pub(crate) columns: Arc<Vec<MySqlColumn>>,
    pub(crate) column_names: Arc<HashMap<UStr, usize>>,
}

impl Row for MySqlRow {
    type Database = MySql;

    fn columns(&self) -> &[MySqlColumn] {
        &self.columns
    }

    fn try_get_raw<I>(&self, index: I) -> Result<MySqlValueRef<'_>, Error>
    where
        I: ColumnIndex<Self>,
    {
        let index = index.index(self)?;
        let column = &self.columns[index];
        let value = self.row.get(index);

        Ok(MySqlValueRef {
            format: self.format,
            row: Some(&self.row.storage),
            type_info: column.type_info.clone(),
            value,
        })
    }
}



impl ColumnIndex<MySqlRow> for &'_ str {
   
    fn index(&self, row: &MySqlRow) -> Result<usize, Error> {

        // Original fast path (works for normal SELECTs)
        if let Some(&idx) = row.column_names.get(*self) {   
            return Ok(idx);
        } else {

        // NEW: Fallback for stored procedures / CALL (your requested change)
        // We scan the real columns and add the name→index mapping on the fly
        for (i, col) in row.columns.iter().enumerate() {
            if  &*col.name == *self {
                // Optional: you could even mutate the map here if you want to "cache" it,
                // but for simplicity we just return the index.

                return Ok(i);
            }        }
        
        Err(Error::ColumnNotFound((*self).into()))
        }
    }
}

impl std::fmt::Debug for MySqlRow {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        debug_row(self, f)
    }
}


