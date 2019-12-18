use crate::{mysql::{protocol::ResultRow, Connection}, row::Row, MySql};

impl Row for ResultRow {
    type Backend = MySql;

    #[inline]
    fn len(&self) -> usize {
        self.values.len()
    }

    #[inline]
    fn get_raw(&self, index: usize) -> Option<&[u8]> {
        self.values[index]
            .as_ref()
            .map(|value| unsafe { value.as_ref() })
    }
}
