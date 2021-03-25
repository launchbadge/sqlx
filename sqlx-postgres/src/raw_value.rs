use bytes::Bytes;
use sqlx_core::RawValue;

use crate::{PgTypeInfo, Postgres};

/// The format of a raw SQL value for Postgres.
///
/// Postgres returns values in [`Text`] or [`Binary`] format with a
/// configuration option in a prepared query. SQLx currently hard-codes that
/// option to [`Binary`].
///
/// For simple queries, postgres only can return values in [`Text`] format.
///
#[derive(Debug, PartialEq, Copy, Clone)]
pub enum PgRawValueFormat {
    Binary,
    Text,
}

/// The raw representation of a SQL value for Postgres.
// 'r: row
#[derive(Debug, Clone)]
#[allow(clippy::module_name_repetitions)]
pub struct PgRawValue<'r> {
    value: Option<&'r Bytes>,
    format: PgRawValueFormat,
    type_info: PgTypeInfo,
}

// 'r: row
impl<'r> PgRawValue<'r> {
    /// Returns the type information for this value.
    #[must_use]
    pub const fn type_info(&self) -> &PgTypeInfo {
        &self.type_info
    }

    /// Returns the format of this value.
    #[must_use]
    pub const fn format(&self) -> PgRawValueFormat {
        self.format
    }
}

impl<'r> RawValue<'r> for PgRawValue<'r> {
    type Database = Postgres;

    fn is_null(&self) -> bool {
        self.value.is_none()
    }

    fn type_info(&self) -> &PgTypeInfo {
        &self.type_info
    }
}
