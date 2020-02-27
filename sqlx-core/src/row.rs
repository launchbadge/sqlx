//! Contains the Row and FromRow traits.

use crate::database::Database;
use crate::decode::Decode;
use crate::types::Type;

pub trait ColumnIndex<'c, R: ?Sized>
where
    R: Row<'c>,
{
    fn try_resolve(self, row: &'c R) -> crate::Result<usize>;
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
        I: ColumnIndex<'c, Self>,
        T: Decode<Self::Database>,
    {
        // todo: use expect with a proper message
        self.try_get(index).unwrap()
    }

    fn try_get<T, I>(&'c self, index: I) -> crate::Result<T>
    where
        T: Type<Self::Database>,
        I: ColumnIndex<'c, Self>,
        T: Decode<Self::Database>,
    {
        Ok(Decode::decode_nullable(self.try_get_raw(index)?)?)
    }

    fn try_get_raw<'i, I>(&'c self, index: I) -> crate::Result<Option<&'c [u8]>>
    where
        I: ColumnIndex<'c, Self> + 'i;
}

/// A **record** that can be built from a row returned from by the database.
pub trait FromRow<'a, R>
where
    R: Row<'a>,
{
    fn from_row(row: R) -> Self;
}

// Macros to help unify the internal implementations as a good chunk is very similar

#[allow(unused_macros)]
macro_rules! impl_map_row_for_row {
    ($DB:ident, $R:ident) => {
        impl<O: Unpin, F> crate::query::MapRow<$DB> for F
        where
            F: for<'c> FnMut($R<'c>) -> crate::Result<O>,
        {
            type Mapped = O;

            fn map_row(&mut self, row: $R) -> crate::Result<O> {
                (self)(row)
            }
        }
    };
}

#[allow(unused_macros)]
macro_rules! impl_column_index_for_row {
    ($R:ident) => {
        impl<'c> crate::row::ColumnIndex<'c, $R<'c>> for usize {
            fn try_resolve(self, row: &'c $R<'c>) -> crate::Result<usize> {
                if self >= row.len() {
                    return Err(crate::Error::ColumnIndexOutOfBounds {
                        len: row.len(),
                        index: self,
                    });
                }

                Ok(self)
            }
        }

        impl<'c> crate::row::ColumnIndex<'c, $R<'c>> for &'_ str {
            fn try_resolve(self, row: &'c $R<'c>) -> crate::Result<usize> {
                row.columns
                    .get(self)
                    .ok_or_else(|| crate::Error::ColumnNotFound((*self).into()))
                    .map(|&index| index)
            }
        }
    };
}

#[allow(unused_macros)]
macro_rules! impl_from_row_for_row {
    ($R:ident) => {
        impl<'c> crate::row::FromRow<'c, $R<'c>> for $R<'c> {
            #[inline]
            fn from_row(row: $R<'c>) -> Self {
                row
            }
        }
    };
}
