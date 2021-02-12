use crate::database::HasRawValue;
use crate::{Database, Decode};

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

    /// Returns the column name, given the ordinal (also known as index) of the column.
    fn column_name_of(&self, ordinal: usize) -> &str;

    /// Returns the column name, given the ordinal (also known as index) of the column.
    fn try_column_name_of(&self, ordinal: usize) -> crate::Result<&str>;

    /// Returns the column ordinal, given the name of the column.
    fn ordinal_of(&self, name: &str) -> usize;

    /// Returns the column ordinal, given the name of the column.
    fn try_ordinal_of(&self, name: &str) -> crate::Result<usize>;

    /// Returns the decoded value at the index.
    fn try_get<'r, T>(&'r self, index: usize) -> crate::Result<T>
    where
        T: Decode<'r, Self::Database>;

    /// Returns the raw representation of the value at the index.
    // noinspection RsNeedlessLifetimes
    fn try_get_raw<'r>(
        &'r self,
        index: usize,
    ) -> crate::Result<<Self::Database as HasRawValue<'r>>::RawValue>;
}

// TODO: fn type_info_of(index)
// TODO: fn try_type_info_of(index)
// TODO: trait ColumnIndex
