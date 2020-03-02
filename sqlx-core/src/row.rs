//! Contains the Row and FromRow traits.

use crate::database::{Database, HasRawValue, HasRow};
use crate::decode::Decode;
use crate::types::Type;

pub trait ColumnIndex<DB>
where
    DB: Database,
    DB: for<'c> HasRow<'c, Database = DB>,
{
    fn resolve<'c>(self, row: &<DB as HasRow<'c>>::Row) -> crate::Result<usize>;
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

    fn get<'r, T, I>(&'r self, index: I) -> crate::Result<T>
    where
        T: Type<Self::Database>,
        I: ColumnIndex<Self::Database>,
        T: Decode<'c, Self::Database>,
    {
        Ok(Decode::decode(self.get_raw(index)?)?)
    }

    fn get_raw<'r, I>(
        &self,
        index: I,
    ) -> crate::Result<<Self::Database as HasRawValue<'c>>::RawValue>
    where
        I: ColumnIndex<Self::Database>;
}

/// A **record** that can be built from a row returned from by the database.
pub trait FromRow<'c, R>
where
    Self: Sized,
    R: Row<'c>,
{
    fn from_row(row: R) -> crate::Result<Self>;
}

// Macros to help unify the internal implementations as a good chunk
// is very similar

#[allow(unused_macros)]
macro_rules! impl_from_row_for_tuple {
    ($db:ident, $r:ident; $( ($idx:tt) -> $T:ident );+;) => {
        impl<'c, $($T,)+> crate::row::FromRow<'c, $r<'c>> for ($($T,)+)
        where
            $($T: crate::types::Type<$db>,)+
            $($T: crate::decode::Decode<'c, $db>,)+
        {
            #[inline]
            fn from_row(row: $r<'c>) -> crate::Result<Self> {
                use crate::row::Row;

                Ok(($(row.get($idx as usize)?,)+))
            }
        }
    };
}

#[allow(unused_macros)]
macro_rules! impl_from_row_for_tuples {
    ($db:ident, $r:ident) => {
        impl_from_row_for_tuple!($db, $r;
            (0) -> T1;
        );

        impl_from_row_for_tuple!($db, $r;
            (0) -> T1;
            (1) -> T2;
        );

        impl_from_row_for_tuple!($db, $r;
            (0) -> T1;
            (1) -> T2;
            (2) -> T3;
        );

        impl_from_row_for_tuple!($db, $r;
            (0) -> T1;
            (1) -> T2;
            (2) -> T3;
            (3) -> T4;
        );

        impl_from_row_for_tuple!($db, $r;
            (0) -> T1;
            (1) -> T2;
            (2) -> T3;
            (3) -> T4;
            (4) -> T5;
        );

        impl_from_row_for_tuple!($db, $r;
            (0) -> T1;
            (1) -> T2;
            (2) -> T3;
            (3) -> T4;
            (4) -> T5;
            (5) -> T6;
        );

        impl_from_row_for_tuple!($db, $r;
            (0) -> T1;
            (1) -> T2;
            (2) -> T3;
            (3) -> T4;
            (4) -> T5;
            (5) -> T6;
            (6) -> T7;
        );

        impl_from_row_for_tuple!($db, $r;
            (0) -> T1;
            (1) -> T2;
            (2) -> T3;
            (3) -> T4;
            (4) -> T5;
            (5) -> T6;
            (6) -> T7;
            (7) -> T8;
        );

        impl_from_row_for_tuple!($db, $r;
            (0) -> T1;
            (1) -> T2;
            (2) -> T3;
            (3) -> T4;
            (4) -> T5;
            (5) -> T6;
            (6) -> T7;
            (7) -> T8;
            (8) -> T9;
        );
    };
}

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
    ($DB:ident) => {
        impl crate::row::ColumnIndex<$DB> for usize {
            fn resolve<'c>(
                self,
                row: &<$DB as crate::database::HasRow<'c>>::Row,
            ) -> crate::Result<usize> {
                let len = crate::row::Row::len(row);

                if self >= len {
                    return Err(crate::Error::ColumnIndexOutOfBounds { len, index: self });
                }

                Ok(self)
            }
        }

        impl crate::row::ColumnIndex<$DB> for &'_ str {
            fn resolve<'c>(
                self,
                row: &<$DB as crate::database::HasRow<'c>>::Row,
            ) -> crate::Result<usize> {
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
            fn from_row(row: $R<'c>) -> crate::Result<Self> {
                Ok(row)
            }
        }
    };
}
