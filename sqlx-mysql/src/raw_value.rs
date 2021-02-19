use std::str::from_utf8;

use bytes::Bytes;
use sqlx_core::decode::{Error as DecodeError, Result as DecodeResult};
use sqlx_core::{Decode, Runtime};

use crate::{MySql, MySqlTypeInfo};

/// The format of a raw SQL value for MySQL.
///
/// MySQL returns values in [`Text`] format for unprepared queries and in [`Binary`]
/// format for prepared queries. There is no facility to request a different format.
///
#[derive(Debug, PartialEq, Copy, Clone)]
pub enum MySqlRawValueFormat {
    Binary,
    Text,
}

/// The raw representation of a SQL value for MySQL.
#[derive(Debug, Clone)]
pub struct MySqlRawValue<'r> {
    value: Option<&'r Bytes>,
    format: MySqlRawValueFormat,
    type_info: &'r MySqlTypeInfo,
}

// 'r: row
impl<'r> MySqlRawValue<'r> {
    pub(crate) fn new(
        value: &'r Option<Bytes>,
        format: MySqlRawValueFormat,
        type_info: &'r MySqlTypeInfo,
    ) -> Self {
        Self { value: value.as_ref(), format, type_info }
    }

    /// Returns the type information for this value.
    pub fn type_info(&self) -> &'r MySqlTypeInfo {
        self.type_info
    }

    /// Returns the format of this value.
    pub const fn format(&self) -> MySqlRawValueFormat {
        self.format
    }

    /// Returns the underlying byte view of this value as a byte slice.
    pub fn as_bytes(&self) -> DecodeResult<&'r [u8]> {
        self.value.map(|bytes| &**bytes).ok_or(DecodeError::UnexpectedNull)
    }

    /// Returns a `&str` slice from the underlying byte view of this value,
    /// if it contains valid UTF-8.
    pub fn as_str(&self) -> DecodeResult<&'r str> {
        self.as_bytes().and_then(|bytes| from_utf8(bytes).map_err(DecodeError::NotUtf8))
    }

    /// Decode this value into the target type.
    pub fn decode<T: Decode<'r, MySql>>(self) -> DecodeResult<T> {
        <T as Decode<'r, MySql>>::decode(self)
    }
}
