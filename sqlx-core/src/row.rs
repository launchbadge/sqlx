//! Contains the Row and FromRow traits.

use crate::database::Database;
use crate::decode::Decode;
use crate::types::HasSqlType;

pub trait RowIndex<R: ?Sized>
where
    R: Row,
{
    fn try_get<T>(&self, row: &R) -> crate::Result<T>
    where
        R::Database: HasSqlType<T>,
        T: Decode<R::Database>;
}

/// Represents a single row of the result set.
pub trait Row: Unpin + Send + 'static {
    type Database: Database + ?Sized;

    /// Returns `true` if the row contains no values.
    fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Returns the number of values in the row.
    fn len(&self) -> usize;

    /// Returns the value at the `index`; can either be an integer ordinal or a column name.
    fn get<T, I>(&self, index: I) -> T
    where
        Self::Database: HasSqlType<T>,
        I: RowIndex<Self>,
        T: Decode<Self::Database>;
}

/// A **record** that can be built from a row returned from by the database.
pub trait FromRow<R>
where
    R: Row,
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
