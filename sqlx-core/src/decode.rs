//! Types and traits for decoding values from the database.

use std::error::Error as StdError;
use std::fmt::{self, Display};

use crate::database::{Database, HasRawValue};

/// Decode a single value from the database.
pub trait Decode<'de, DB>
where
    Self: Sized + 'de,
    DB: HasRawValue<'de>,
{
    fn decode(value: DB::RawValue) -> crate::Result<Self>;
}
