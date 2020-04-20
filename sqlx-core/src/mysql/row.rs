use std::sync::Arc;

use hashbrown::HashMap;

use crate::error::Error;
use crate::ext::ustr::UStr;
use crate::mysql::{protocol, MySql, MySqlTypeInfo, MySqlValueFormat, MySqlValueRef};
use crate::row::{ColumnIndex, Row};

// TODO: Merge with the other XXColumn types
#[derive(Debug, Clone)]
pub(crate) struct MySqlColumn {
    pub(crate) name: Option<UStr>,
    pub(crate) type_info: Option<MySqlTypeInfo>,
}

/// Implementation of [`Row`] for MySQL.
#[derive(Debug)]
pub struct MySqlRow {
    pub(crate) row: protocol::Row,
    pub(crate) columns: Arc<Vec<MySqlColumn>>,
    pub(crate) column_names: Arc<HashMap<UStr, usize>>,
    pub(crate) format: MySqlValueFormat,
}

impl crate::row::private_row::Sealed for MySqlRow {}

impl Row for MySqlRow {
    type Database = MySql;

    #[inline]
    fn len(&self) -> usize {
        self.row.len()
    }

    fn try_get_raw<I>(&self, index: I) -> Result<MySqlValueRef, Error>
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
        row.column_names
            .get(*self)
            .ok_or_else(|| Error::ColumnNotFound((*self).into()))
            .map(|v| *v)
    }
}
