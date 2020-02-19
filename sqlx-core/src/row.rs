//! Contains the Row and FromRow traits.

use crate::database::Database;
use crate::decode::Decode;
use crate::types::Type;

pub trait RowIndex<'c, R: ?Sized>
where
    R: Row<'c>,
{
    fn try_get_raw(self, row: &'c R) -> crate::Result<Option<&'c [u8]>>;
}

/// Represents a single row of the result set.
pub trait Row<'c>: Unpin + Send {
    type Database: Database + ?Sized;

    /// Returns `true` if the row contains no values.
    fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Returns the number of values in the row.
    fn len(&self) -> usize;

    fn get<T, I>(&'c self, index: I) -> T
    where
        T: Type<Self::Database>,
        I: RowIndex<'c, Self>,
        T: Decode<Self::Database>,
    {
        // todo: use expect with a proper message
        self.try_get(index).unwrap()
    }

    fn try_get<T, I>(&'c self, index: I) -> crate::Result<T>
    where
        T: Type<Self::Database>,
        I: RowIndex<'c, Self>,
        T: Decode<Self::Database>,
    {
        Ok(Decode::decode_nullable(self.try_get_raw(index)?)?)
    }

    fn try_get_raw<'i, I>(&'c self, index: I) -> crate::Result<Option<&'c [u8]>>
    where
        I: RowIndex<'c, Self> + 'i;
}

/// A **record** that can be built from a row returned from by the database.
pub trait FromRow<'a, R>
where
    R: Row<'a>,
{
    fn from_row(row: R) -> Self;
}

#[allow(unused_macros)]
macro_rules! impl_from_row_for_row {
    ($R:ty) => {
        impl crate::row::FromRow<$R> for $R {
            #[inline]
            fn from_row(row: $R) -> Self {
                row
            }
        }
    };
}
