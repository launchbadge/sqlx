use std::collections::HashMap;
use std::sync::Arc;

use crate::connection::MaybeOwnedConnection;
use crate::decode::Decode;
use crate::pool::PoolConnection;
use crate::postgres::protocol::DataRow;
use crate::postgres::{PgConnection, Postgres};
use crate::row::{Row, RowIndex};
use crate::types::Type;

pub struct PgRow<'c> {
    pub(super) connection: MaybeOwnedConnection<'c, PgConnection>,
    pub(super) data: DataRow,
    pub(super) columns: Arc<HashMap<Box<str>, usize>>,
}

impl<'c> Row<'c> for PgRow<'c> {
    type Database = Postgres;

    fn len(&self) -> usize {
        self.data.len()
    }

    fn try_get_raw<'i, I>(&'c self, index: I) -> crate::Result<Option<&'c [u8]>>
    where
        I: RowIndex<'c, Self> + 'i,
    {
        index.try_get_raw(self)
    }
}

impl<'c> RowIndex<'c, PgRow<'c>> for usize {
    fn try_get_raw(self, row: &'c PgRow<'c>) -> crate::Result<Option<&'c [u8]>> {
        Ok(row.data.get(
            row.connection.stream.buffer(),
            &row.connection.data_row_values_buf,
            self,
        ))
    }
}

impl<'c> RowIndex<'c, PgRow<'c>> for &'_ str {
    fn try_get_raw(self, row: &'c PgRow<'c>) -> crate::Result<Option<&'c [u8]>> {
        let index = row
            .columns
            .get(self)
            .ok_or_else(|| crate::Error::ColumnNotFound((*self).into()))?;

        Ok(row.data.get(
            row.connection.stream.buffer(),
            &row.connection.data_row_values_buf,
            *index,
        ))
    }
}

// TODO: impl_from_row_for_row!(PgRow);
