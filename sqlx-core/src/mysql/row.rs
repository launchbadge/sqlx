use std::collections::HashMap;
use std::sync::Arc;
use std::vec::Vec;

use crate::mysql::protocol;
use crate::mysql::{MySql, MySqlValue};
use crate::row::{Column, ColumnIndex, Row};

pub struct MySqlRow<'c> {
    pub(super) row: protocol::Row<'c>,
    pub(super) names: Arc<HashMap<Box<str>, u16>>,
}

impl crate::row::private_row::Sealed for MySqlRow<'_> {}

impl<'c> Row<'c> for MySqlRow<'c> {
    type Database = MySql;

    fn len(&self) -> usize {
        self.row.len()
    }

    #[doc(hidden)]
    fn try_get_raw<I>(&self, index: I) -> crate::Result<MySqlValue<'c>>
    where
        I: ColumnIndex<'c, Self>,
    {
        let index = index.index(self)?;
        let column_ty = self.row.columns[index].clone();
        let buffer = self.row.get(index);
        let value = match (self.row.binary, buffer) {
            (_, None) => MySqlValue::null(),
            (true, Some(buf)) => MySqlValue::binary(column_ty, buf),
            (false, Some(buf)) => MySqlValue::text(column_ty, buf),
        };

        Ok(value)
    }

    fn columns(&self) -> Box<[Column<Self::Database>]> {
        let mut columns = Vec::with_capacity(self.row.columns.len());
        for (index, column_type) in self.row.columns.iter().enumerate() {
            let name = self
                .names
                .iter()
                .find(|(_name, name_index)| (**name_index as usize) == index)
                .map(|(name, _)| name.as_ref());

            columns.push(Column {
                name,
                type_info: Some(column_type),
            });
        }

        columns.into_boxed_slice()
    }
}

impl<'c> ColumnIndex<'c, MySqlRow<'c>> for usize {
    fn index(&self, row: &MySqlRow<'c>) -> crate::Result<usize> {
        let len = Row::len(row);

        if *self >= len {
            return Err(crate::Error::ColumnIndexOutOfBounds { len, index: *self });
        }

        Ok(*self)
    }
}

impl<'c> ColumnIndex<'c, MySqlRow<'c>> for str {
    fn index(&self, row: &MySqlRow<'c>) -> crate::Result<usize> {
        row.names
            .get(self)
            .ok_or_else(|| crate::Error::ColumnNotFound((*self).into()))
            .map(|&index| index as usize)
    }
}
