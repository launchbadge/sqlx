use std::error::Error as StdError;

use crate::database::{Database, HasRawValue};
use crate::value::RawValue;

// alias as this is a long type that's used everywhere
pub(crate) type Error = Box<dyn StdError + 'static + Send + Sync>;

/// A type that can be decoded from the database.
pub trait Decode<'r, DB: Database>: Sized {
    // TODO: [accepts] would replace our massive switch in [Type::compatible]
    // fn accepts(info: &DB::TypeInfo) -> bool;

    fn decode(value: <DB as HasRawValue<'r>>::RawValue) -> Result<Self, Error>;
}

/// An unexpected `NULL` was encountered during decoding.
///
/// Returned from [`Row::get`] if the value from the database is `NULL`
/// and you are not decoding into an `Option`.
#[derive(thiserror::Error, Debug)]
#[error("unexpected null; try decoding as an `Option`")]
pub struct UnexpectedNullError;
