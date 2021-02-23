use crate::database::HasRawValue;
use crate::{Database, Decode};

/// A single row from a result set generated from the database.
pub trait Row: 'static + Send + Sync {
    type Database: Database;

    /// Returns `true` if the row contains only `NULL` values.
    fn is_null(&self) -> bool;

    /// Returns the number of columns in the row.
    fn len(&self) -> usize;

    /// Returns `true` if there are no columns in the row.
    fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Returns a reference to the columns in the row.
    fn columns(&self) -> &[<Self::Database as Database>::Column];

    /// Returns the column at the index, if available.
    fn column<I: ColumnIndex<Self>>(&self, index: I) -> &<Self::Database as Database>::Column;

    /// Returns the column at the index, if available.
    fn try_column<I: ColumnIndex<Self>>(
        &self,
        index: I,
    ) -> crate::Result<&<Self::Database as Database>::Column>;

    /// Returns the column name, given the index of the column.
    fn column_name_of(&self, index: usize) -> &str;

    /// Returns the column name, given the index of the column.
    fn try_column_name_of(&self, index: usize) -> crate::Result<&str>;

    /// Returns the column index, given the name of the column.
    fn index_of(&self, name: &str) -> usize;

    /// Returns the column index, given the name of the column.
    fn try_index_of(&self, name: &str) -> crate::Result<usize>;

    /// Returns the decoded value at the index.
    fn get<'r, T, I>(&'r self, index: I) -> T
    where
        I: ColumnIndex<Self>,
        T: Decode<'r, Self::Database>;

    /// Returns the decoded value at the index.
    fn try_get<'r, T, I>(&'r self, index: I) -> crate::Result<T>
    where
        I: ColumnIndex<Self>,
        T: Decode<'r, Self::Database>;

    /// Returns the raw representation of the value at the index.
    #[allow(clippy::needless_lifetimes)]
    fn get_raw<'r, I: ColumnIndex<Self>>(
        &'r self,
        index: I,
    ) -> <Self::Database as HasRawValue<'r>>::RawValue;

    /// Returns the raw representation of the value at the index.
    #[allow(clippy::needless_lifetimes)]
    fn try_get_raw<'r, I: ColumnIndex<Self>>(
        &'r self,
        index: I,
    ) -> crate::Result<<Self::Database as HasRawValue<'r>>::RawValue>;
}

/// A helper trait used for indexing into a [`Row`].
pub trait ColumnIndex<R: Row + ?Sized> {
    /// Returns the index of the column at this index, if present.
    #[allow(clippy::needless_lifetimes)]
    fn get<'r>(&self, row: &'r R) -> crate::Result<usize>;
}

// access by index
impl<R: Row> ColumnIndex<R> for usize {
    #[allow(clippy::needless_lifetimes)]
    fn get<'r>(&self, _row: &'r R) -> crate::Result<usize> {
        // note: the "index out of bounds" error will be surfaced
        //  by [try_get]
        Ok(*self)
    }
}

// access by name
impl<R: Row> ColumnIndex<R> for &'_ str {
    #[allow(clippy::needless_lifetimes)]
    fn get<'r>(&self, row: &'r R) -> crate::Result<usize> {
        row.try_index_of(self)
    }
}

// access by reference
impl<R: Row, I: ColumnIndex<R>> ColumnIndex<R> for &'_ I {
    #[allow(clippy::needless_lifetimes)]
    fn get<'r>(&self, row: &'r R) -> crate::Result<usize> {
        (*self).get(row)
    }
}
