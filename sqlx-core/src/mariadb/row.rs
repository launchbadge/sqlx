use crate::{
    mariadb::{protocol::ResultRow, MariaDb},
    row::RawRow,
};

pub struct MariaDbRow(pub(super) ResultRow);

impl RawRow for MariaDbRow {
    type Backend = MariaDb;

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
