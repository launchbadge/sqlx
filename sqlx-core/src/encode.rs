//! Types and traits for encoding values to the database.
use std::mem;

use crate::database::{Database, HasArguments};
use crate::types::Type;

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
pub trait Encode<'q, DB: Database>: Type<DB> {
    fn produces(&self) -> DB::TypeInfo {
        Self::type_info()
    }

    /// Writes the value of `self` into `buf` in the expected format for the database.
    #[must_use]
    fn encode(self, buf: &mut <DB as HasArguments<'q>>::ArgumentBuffer) -> IsNull
    where
        Self: Sized,
    {
        self.encode_by_ref(buf)
    }

    /// Writes the value of `self` into `buf` without moving `self`.
    ///
    /// Where possible, make use of `encode` instead as it can take advantage of re-using
    /// memory.
    #[must_use]
    fn encode_by_ref(&self, buf: &mut <DB as HasArguments<'q>>::ArgumentBuffer) -> IsNull;

    #[inline]
    fn size_hint(&self) -> usize {
        mem::size_of_val(self)
    }
}

impl<'q, T, DB: Database> Encode<'q, DB> for &'_ T
where
    T: Encode<'q, DB>,
{
    #[inline]
    fn produces(&self) -> DB::TypeInfo {
        (**self).produces()
    }

    #[inline]
    fn encode(self, buf: &mut <DB as HasArguments<'q>>::ArgumentBuffer) -> IsNull {
        <T as Encode<DB>>::encode_by_ref(self, buf)
    }

    #[inline]
    fn encode_by_ref(&self, buf: &mut <DB as HasArguments<'q>>::ArgumentBuffer) -> IsNull {
        <&T as Encode<DB>>::encode(self, buf)
    }

    #[inline]
    fn size_hint(&self) -> usize {
        (**self).size_hint()
    }
}

impl<'q, T: 'q + Encode<'q, DB>, DB: Database> Encode<'q, DB> for Option<T> {
    #[inline]
    fn produces(&self) -> DB::TypeInfo {
        if let Some(v) = self {
            v.produces()
        } else {
            T::type_info()
        }
    }

    #[inline]
    fn encode(self, buf: &mut <DB as HasArguments<'q>>::ArgumentBuffer) -> IsNull {
        if let Some(v) = self {
            v.encode(buf)
        } else {
            IsNull::Yes
        }
    }

    #[inline]
    fn encode_by_ref(&self, buf: &mut <DB as HasArguments<'q>>::ArgumentBuffer) -> IsNull {
        if let Some(v) = self {
            v.encode_by_ref(buf)
        } else {
            IsNull::Yes
        }
    }

    #[inline]
    fn size_hint(&self) -> usize {
        self.as_ref().map_or(0, Encode::size_hint)
    }
}
