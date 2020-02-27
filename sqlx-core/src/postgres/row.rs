use std::collections::HashMap;
use std::sync::Arc;

use crate::connection::MaybeOwnedConnection;
use crate::decode::Decode;
use crate::pool::PoolConnection;
use crate::postgres::protocol::DataRow;
use crate::postgres::{PgConnection, Postgres};
use crate::row::{ColumnIndex, Row};
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
        I: ColumnIndex<'c, Self> + 'i,
    {
        Ok(self.data.get(
            self.connection.stream.buffer(),
            &self.connection.current_row_values,
            index.try_resolve(self)?,
        ))
    }
}

impl_map_row_for_row!(Postgres, PgRow);
impl_column_index_for_row!(PgRow);
impl_from_row_for_row!(PgRow);
