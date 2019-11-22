//! Types and traits related to serializing values for the database.
use crate::{backend::Backend, types::HasSqlType};

/// Annotates the result of [Encode] to differentiate between an empty value and a null value.
pub enum IsNull {
    /// The value was null (and no data was written to the buffer).
    Yes,

    /// The value was not null.
    ///
    /// This does not necessarily mean that any data was written to the buffer.
    No,
}

/// Serializes a single value to be sent to the database.
///
/// The data must be written to the buffer in the expected format
/// for the given backend.
///
/// When possible, implementations of this trait should prefer using an
/// existing implementation, rather than writing to `buf` directly.
pub trait Encode<DB: Backend> {
    /// Writes the value of `self` into `buf` as the expected format
    /// for the given backend.
    ///
    /// The return value indicates if this value should be represented as `NULL`.
    /// If this is the case, implementations **must not** write anything to `out`.
    fn encode(&self, buf: &mut Vec<u8>) -> IsNull;
}

/// [Encode] is implemented for `Option<T>` where `T` implements `Encode`. An `Option<T>`
/// represents a nullable SQL value.
impl<T, DB> Encode<DB> for Option<T>
where
    DB: Backend + HasSqlType<T>,
    T: Encode<DB>,
{
    #[inline]
    fn encode(&self, buf: &mut Vec<u8>) -> IsNull {
        if let Some(self_) = self {
            self_.encode(buf)
        } else {
            IsNull::Yes
        }
    }
}

impl<T: ?Sized, DB> Encode<DB> for &'_ T
where
    DB: Backend + HasSqlType<T>,
    T: Encode<DB>,
{
    #[inline]
    fn encode(&self, buf: &mut Vec<u8>) -> IsNull {
        (*self).encode(buf)
    }
}
