use std::str::from_utf8;

use bytes::Bytes;
use sqlx_core::decode::{Error as DecodeError, Result as DecodeResult};
use sqlx_core::{RawValue, Result};

use crate::{PgClientError, PgTypeInfo, Postgres};

/// The format of a raw SQL value for Postgres.
///
/// Postgres returns values in [`Text`] or [`Binary`] format with a
/// configuration option in a prepared query. SQLx currently hard-codes that
/// option to [`Binary`].
///
/// For simple queries, postgres only can return values in [`Text`] format.
///
#[derive(Debug, PartialEq, Copy, Clone)]
#[repr(i16)]
pub enum PgRawValueFormat {
    Text = 0,
    Binary = 1,
}

impl PgRawValueFormat {
    pub(crate) fn from_i16(value: i16) -> Result<Self> {
        match value {
            0 => Ok(Self::Text),
            1 => Ok(Self::Binary),

            _ => Err(PgClientError::UnknownValueFormat(value).into()),
        }
    }
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

impl<'r> PgRawValue<'r> {
    pub(crate) fn new(
        value: &'r Option<Bytes>,
        format: PgRawValueFormat,
        type_info: PgTypeInfo,
    ) -> Self {
        Self { value: value.as_ref(), format, type_info }
    }

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

    /// Returns the underlying byte view of this value.
    pub fn as_bytes(&self) -> DecodeResult<&'r [u8]> {
        self.value.map(|bytes| &**bytes).ok_or(DecodeError::UnexpectedNull)
    }

    /// Returns a `&str` slice from the underlying byte view of this value,
    /// if it contains valid UTF-8.
    pub fn as_str(&self) -> DecodeResult<&'r str> {
        self.as_bytes().and_then(|bytes| from_utf8(bytes).map_err(DecodeError::NotUtf8))
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
