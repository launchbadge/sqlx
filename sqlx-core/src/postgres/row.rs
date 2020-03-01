use core::str::{from_utf8, Utf8Error};

use std::collections::HashMap;
use std::convert::TryFrom;
use std::sync::Arc;

use crate::connection::MaybeOwnedConnection;
use crate::decode::Decode;
use crate::error::UnexpectedNullError;
use crate::pool::PoolConnection;
use crate::postgres::protocol::{DataRow, TypeFormat};
use crate::postgres::{PgConnection, Postgres};
use crate::row::{ColumnIndex, Row};
use crate::types::Type;

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
    pub(super) connection: MaybeOwnedConnection<'c, PgConnection>,
    pub(super) data: DataRow,
    pub(super) columns: Arc<HashMap<Box<str>, usize>>,
    pub(super) formats: Arc<[TypeFormat]>,
}

impl<'c> Row<'c> for PgRow<'c> {
    type Database = Postgres;

    fn len(&self) -> usize {
        self.data.len()
    }

    fn get_raw<'i, I>(&'c self, index: I) -> crate::Result<Option<PgValue<'c>>>
    where
        I: ColumnIndex<'c, Self> + 'i,
    {
        let index = index.resolve(self)?;

        self.data
            .get(
                self.connection.stream.buffer(),
                &self.connection.current_row_values,
                index,
            )
            .map(|buf| match self.formats[index] {
                TypeFormat::Binary => Ok(PgValue::Binary(buf)),
                TypeFormat::Text => Ok(PgValue::Text(from_utf8(buf)?)),
            })
            .transpose()
            .map_err(|err: Utf8Error| crate::Error::Decode(Box::new(err)))
    }
}

impl_map_row_for_row!(Postgres, PgRow);
impl_column_index_for_row!(PgRow);
impl_from_row_for_row!(PgRow);
