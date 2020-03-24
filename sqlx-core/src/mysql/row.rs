use std::collections::HashMap;
use std::convert::TryFrom;
use std::sync::Arc;

use crate::error::UnexpectedNullError;
use crate::mysql::protocol;
use crate::mysql::MySql;
use crate::row::{ColumnIndex, Row};

#[derive(Debug)]
pub enum MySqlValue<'c> {
    Binary(&'c [u8]),
    Text(&'c [u8]),
}

impl<'c> TryFrom<Option<MySqlValue<'c>>> for MySqlValue<'c> {
    type Error = crate::Error<MySql>;

    #[inline]
    fn try_from(value: Option<MySqlValue<'c>>) -> Result<Self, Self::Error> {
        match value {
            Some(value) => Ok(value),
            None => Err(crate::Error::<MySql>::decode(UnexpectedNullError)),
        }
    }
}

pub struct MySqlRow<'c> {
    pub(super) row: protocol::Row<'c>,
    pub(super) columns: Arc<HashMap<Box<str>, u16>>,
}

impl<'c> Row<'c> for MySqlRow<'c> {
    type Database = MySql;

    fn len(&self) -> usize {
        self.row.len()
    }

    fn try_get_raw<I>(&self, index: I) -> crate::Result<MySql, Option<MySqlValue<'c>>>
    where
        I: ColumnIndex<'c, Self>,
    {
        let index = index.resolve(self)?;

        Ok(self.row.get(index).map(|buf| {
            if self.row.binary {
                MySqlValue::Binary(buf)
            } else {
                MySqlValue::Text(buf)
            }
        }))
    }
}
