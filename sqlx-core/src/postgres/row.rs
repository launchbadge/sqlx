use core::str::{from_utf8, Utf8Error};

use std::collections::HashMap;
use std::convert::TryFrom;
use std::sync::Arc;

use crate::error::UnexpectedNullError;
use crate::postgres::protocol::{DataRow, TypeFormat};
use crate::postgres::Postgres;
use crate::row::{ColumnIndex, Row};

/// A value from Postgres. This may be in a BINARY or TEXT format depending
/// on the data type and if the query was prepared or not.
pub enum PgValue<'c> {
    Binary(&'c [u8]),
    Text(&'c str),
}

impl<'c> TryFrom<Option<PgValue<'c>>> for PgValue<'c> {
    type Error = crate::Error;

    #[inline]
    fn try_from(value: Option<PgValue<'c>>) -> Result<Self, Self::Error> {
        match value {
            Some(value) => Ok(value),
            None => Err(crate::Error::decode(UnexpectedNullError)),
        }
    }
}

pub struct PgRow<'c> {
    pub(super) data: DataRow<'c>,
    pub(super) columns: Arc<HashMap<Box<str>, usize>>,
    pub(super) formats: Arc<[TypeFormat]>,
}

impl<'c> Row<'c> for PgRow<'c> {
    type Database = Postgres;

    fn len(&self) -> usize {
        self.data.len()
    }

    fn try_get_raw<'r, I>(&'r self, index: I) -> crate::Result<Option<PgValue<'c>>>
    where
        I: ColumnIndex<Self::Database>,
    {
        let index = index.resolve(self)?;
        let buffer = self.data.get(index);

        buffer
            .map(|buf| match self.formats[index] {
                TypeFormat::Binary => Ok(PgValue::Binary(buf)),
                TypeFormat::Text => Ok(PgValue::Text(from_utf8(buf)?)),
            })
            .transpose()
            .map_err(|err: Utf8Error| crate::Error::Decode(Box::new(err)))
    }
}
