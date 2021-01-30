use bytestring::ByteString;
use sqlx_core::Column;

#[allow(clippy::module_name_repetitions)]
pub struct MySqlColumn {
    ordinal: usize,
    name: ByteString,
}

impl MySqlColumn {
    pub fn name(&self) -> &str {
        &self.name
    }

    pub const fn ordinal(&self) -> usize {
        self.ordinal
    }
}

impl Column for MySqlColumn {
    #[inline]
    fn name(&self) -> &str {
        self.name()
    }

    #[inline]
    fn ordinal(&self) -> usize {
        self.ordinal()
    }
}
