//! Types and traits for decoding values from the database.

use std::error::Error as StdError;
use std::fmt::{self, Display};

use crate::database::{Database, HasRawValue};

/// Decode a single value from the database.
pub trait Decode<'de, DB>
where
    Self: Sized,
    DB: HasRawValue<'de>,
{
    fn decode(raw: DB::RawValue) -> crate::Result<Self>;
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
