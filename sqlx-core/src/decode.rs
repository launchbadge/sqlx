//! Types and traits for decoding values from the database.

use crate::database::{Database, HasValueRef};
use crate::error::BoxDynError;
use crate::types::Type;
use crate::value::ValueRef;

/// A type that can be decoded from the database.
pub trait Decode<'r, DB: Database>: Sized + Type<DB> {
    /// Determines if a value of this type can be created from a value with the
    /// given type information.
    fn accepts(ty: &DB::TypeInfo) -> bool {
        *ty == Self::type_info()
    }

    /// Decode a new value of this type using a raw value from the database.
    fn decode(value: <DB as HasValueRef<'r>>::ValueRef) -> Result<Self, BoxDynError>;
}

// implement `Decode` for Option<T> for all SQL types
impl<'r, DB, T> Decode<'r, DB> for Option<T>
where
    DB: Database,
    T: Decode<'r, DB>,
{
    fn accepts(ty: &DB::TypeInfo) -> bool {
        T::accepts(ty)
    }

    fn decode(value: <DB as HasValueRef<'r>>::ValueRef) -> Result<Self, BoxDynError> {
        if value.is_null() {
            Ok(None)
        } else {
            Ok(Some(T::decode(value)?))
        }
    }
}
