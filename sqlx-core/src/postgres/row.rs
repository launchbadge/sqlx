use super::{protocol::DataRow, Postgres};
use crate::row::RawRow;

#[derive(Debug)]
pub struct PostgresRow(pub(crate) DataRow);

impl RawRow for PostgresRow {
    type Backend = Postgres;

    #[inline]
    fn len(&self) -> usize {
        self.0.len()
    }

    #[inline]
    fn get_raw(&self, index: usize) -> Option<&[u8]> {
        self.0.get(index)
    }
}
