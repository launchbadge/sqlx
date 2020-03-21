//! Types and traits for decoding values from the database.

use crate::database::{Database, HasRawValue};

/// A type that can be decoded from the database.
pub trait Decode<'de, DB>
where
    Self: Sized + 'de,
    DB: HasRawValue<'de>,
{
    fn decode(value: DB::RawValue) -> crate::Result<Self>;
}

/// A type that can be decoded without borrowing from the connection.
pub trait DecodeOwned<DB: Database>: for<'de> Decode<'de, DB> {}

impl<DB, T> DecodeOwned<DB> for T
where
    DB: Database,
    T: 'static,
    T: for<'de> Decode<'de, DB>,
{
}
