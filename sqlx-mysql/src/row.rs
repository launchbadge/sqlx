use sqlx_core::Row;

use crate::{protocol, MySqlColumn};

#[allow(clippy::module_name_repetitions)]
pub struct MySqlRow(pub(crate) protocol::Row);

impl MySqlRow {
    #[must_use]
    pub fn len(&self) -> usize {
        self.0.values.len()
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

impl Row for MySqlRow {
    type Column = MySqlColumn;

    fn is_null(&self) -> bool {
        todo!()
    }

    fn len(&self) -> usize {
        self.len()
    }

    fn columns(&self) -> &[MySqlColumn] {
        todo!()
    }

    fn column_name_of(&self, ordinal: usize) -> &str {
        todo!()
    }

    fn try_column_name_of(&self, ordinal: usize) -> sqlx_core::Result<&str> {
        todo!()
    }

    fn ordinal_of(&self, name: &str) -> usize {
        todo!()
    }

    fn try_ordinal_of(&self, name: &str) -> sqlx_core::Result<usize> {
        todo!()
    }

    fn try_get_raw(&self) -> sqlx_core::Result<&[u8]> {
        todo!()
    }
}
