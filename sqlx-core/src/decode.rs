//! Types and traits for decoding values from the database.

use crate::database::{Database, HasRawValue};

/// A type that can be decoded from the database.
pub trait Decode<'de, DB>
where
    Self: Sized + 'de,
    DB: Database,
{
    fn decode(value: <DB as HasRawValue<'de>>::RawValue) -> crate::Result<DB, Self>;
}
