use std::mem;

use crate::database::Database;

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
pub trait Encode<DB: Database> {
    fn produces() -> DB::TypeInfo;

    /// Writes the value of `self` into `buf` in the expected format for the database.
    #[must_use]
    fn encode(&self, buf: &mut DB::RawBuffer) -> IsNull;

    #[inline]
    fn size_hint(&self) -> usize {
        mem::size_of_val(self)
    }
}

impl<T: Encode<DB> + ?Sized, DB: Database> Encode<DB> for &'_ T {
    #[inline]
    fn produces() -> DB::TypeInfo {
        <T as Encode<DB>>::produces()
    }

    #[inline]
    fn encode(&self, buf: &mut DB::RawBuffer) -> IsNull {
        (*self).encode(buf)
    }

    #[inline]
    fn size_hint(&self) -> usize {
        (*self).size_hint()
    }
}

impl<T: Encode<DB>, DB: Database> Encode<DB> for Option<T> {
    #[inline]
    fn produces() -> DB::TypeInfo {
        <T as Encode<DB>>::produces()
    }

    #[inline]
    fn encode(&self, buf: &mut DB::RawBuffer) -> IsNull {
        if let Some(v) = self {
            v.encode(buf);

            IsNull::No
        } else {
            IsNull::Yes
        }
    }

    #[inline]
    fn size_hint(&self) -> usize {
        self.as_ref().map_or(0, Encode::size_hint)
    }
}
