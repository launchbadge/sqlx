//! Types and traits for decoding values from the database.

use std::error::Error as StdError;
use std::fmt::{self, Display};

use crate::database::Database;
use crate::types::Type;

/// Decode a single value from the database.
pub trait Decode<'de, DB>
where
    Self: Sized,
    DB: Database,
{
    fn decode(raw: &'de [u8]) -> crate::Result<Self>;

    /// Creates a new value of this type from a `NULL` SQL value.
    ///
    /// The default implementation returns [DecodeError::UnexpectedNull].
    fn decode_null() -> crate::Result<Self> {
        Err(crate::Error::Decode(UnexpectedNullError.into()))
    }

    fn decode_nullable(raw: Option<&'de [u8]>) -> crate::Result<Self> {
        if let Some(raw) = raw {
            Self::decode(raw)
        } else {
            Self::decode_null()
        }
    }
}

impl<'de, T, DB> Decode<'de, DB> for Option<T>
where
    DB: Database,
    T: Type<DB>,
    T: Decode<'de, DB>,
{
    fn decode(buf: &'de [u8]) -> crate::Result<Self> {
        T::decode(buf).map(Some)
    }

    fn decode_null() -> crate::Result<Self> {
        Ok(None)
    }
}

/// An unexpected `NULL` was encountered during decoding.
///
/// Returned from `Row::try_get` if the value from the database is `NULL`
/// and you are not decoding into an `Option`.
#[derive(Debug, Clone, Copy)]
pub struct UnexpectedNullError;

impl Display for UnexpectedNullError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("unexpected null; try decoding as an `Option`")
    }
}

impl StdError for UnexpectedNullError {}
