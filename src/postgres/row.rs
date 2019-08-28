use super::{protocol::DataRow, Postgres};
use crate::row::Row;

pub struct PostgresRow(pub(crate) DataRow);

impl Row for PostgresRow {
    type Backend = Postgres;

    #[inline]
    fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    #[inline]
    fn len(&self) -> usize {
        self.0.len()
    }

    #[inline]
    fn get_raw(&self, index: usize) -> Option<&[u8]> {
        self.0.get(index)
    }
}
