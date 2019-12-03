use super::{protocol::DataRow, Postgres};
use crate::row::Row;

impl Row for DataRow {
    type Backend = Postgres;

    fn len(&self) -> usize {
        self.values.len()
    }

    fn get_raw(&self, index: usize) -> Option<&[u8]> {
        self.values[index]
            .as_ref()
            .map(|value| unsafe { value.as_ref() })
    }
}
