//! Types and traits for encoding values to the database.

use crate::database::Database;
use crate::types::Type;
use std::mem;

/// The return type of [Encode::encode].
pub enum IsNull {
    /// The value is null; no data was written.
    Yes,

    /// The value is not null.
    ///
    /// This does not mean that data was written.
    No,
}

/// Encode a single value to be sent to the database.
pub trait Encode<DB>
where
    DB: Database + ?Sized,
{
    /// Writes the value of `self` into `buf` in the expected format for the database.
    fn encode(&self, buf: &mut DB::RawBuffer);

    fn encode_nullable(&self, buf: &mut DB::RawBuffer) -> IsNull {
        self.encode(buf);

        IsNull::No
    }

    fn size_hint(&self) -> usize {
        mem::size_of_val(self)
    }
}

impl<T: ?Sized, DB> Encode<DB> for &'_ T
where
    DB: Database,
    T: Type<DB>,
    T: Encode<DB>,
{
    fn encode(&self, buf: &mut DB::RawBuffer) {
        (*self).encode(buf)
    }

    fn encode_nullable(&self, buf: &mut DB::RawBuffer) -> IsNull {
        (*self).encode_nullable(buf)
    }

    fn size_hint(&self) -> usize {
        (*self).size_hint()
    }
}

impl<T, DB> Encode<DB> for Option<T>
where
    DB: Database,
    T: Type<DB>,
    T: Encode<DB>,
{
    fn encode(&self, buf: &mut DB::RawBuffer) {
        // Forward to [encode_nullable] and ignore the result
        let _ = self.encode_nullable(buf);
    }

    fn encode_nullable(&self, buf: &mut DB::RawBuffer) -> IsNull {
        if let Some(self_) = self {
            self_.encode(buf);

            IsNull::No
        } else {
            IsNull::Yes
        }
    }

    fn size_hint(&self) -> usize {
        self.as_ref().map_or(0, Encode::size_hint)
    }
}
