use super::{protocol::DataRow, Pg};
use crate::row::Row;

pub struct PgRow(pub(crate) Box<DataRow>);

impl Row for PgRow {
    type Backend = Pg;

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
