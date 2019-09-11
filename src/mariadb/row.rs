use crate::row::Row;
use crate::mariadb::protocol::ResultRow;
use crate::mariadb::MariaDb;

pub struct MariaDbRow(pub(super) ResultRow);

impl Row for MariaDbRow {
    type Backend = MariaDb;

    #[inline]
    fn is_empty(&self) -> bool {
        self.0.values.is_empty()
    }

    #[inline]
    fn len(&self) -> usize {
        self.0.values.len()
    }

    #[inline]
    fn get_raw(&self, index: usize) -> Option<&[u8]> {
        self.0.values[index]
            .as_ref()
            .map(|value| unsafe { value.as_ref() })
    }
}
