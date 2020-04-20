use crate::decode::{Decode, Error};
use crate::postgres::{PgRawValue, Postgres};

mod array;
mod bool;
mod int;
mod str;

pub use array::PgArrayElement;

// implement `Decode` for all postgres types
// the concept of a nullable `RawValue` is db-specific
impl<'de, T> Decode<'de, Postgres> for Option<T>
where
    T: Decode<'de, Postgres>,
{
    fn decode(value: PgRawValue<'de>) -> Result<Self, Error> {
        if value.is_null() {
            Ok(None)
        } else {
            Ok(Some(T::decode(value)?))
        }
    }
}
