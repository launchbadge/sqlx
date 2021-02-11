use crate::{Column, Database, Runtime};

pub trait Row: 'static + Send + Sync {
    type Column: Column;

    /// Returns `true` if the row contains only `NULL` values.
    fn is_null(&self) -> bool;

    /// Returns the number of columns in the row.
    fn len(&self) -> usize;

    /// Returns `true` if there are no columns in the row.
    fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Returns a reference to the columns in the row.
    fn columns(&self) -> &[Self::Column];

    /// Returns the column name, given the ordinal (also known as index) of the column.
    fn column_name_of(&self, ordinal: usize) -> &str;

    /// Returns the column name, given the ordinal (also known as index) of the column.
    fn try_column_name_of(&self, ordinal: usize) -> crate::Result<&str>;

    /// Returns the column ordinal, given the name of the column.
    fn ordinal_of(&self, name: &str) -> usize;

    /// Returns the column ordinal, given the name of the column.
    fn try_ordinal_of(&self, name: &str) -> crate::Result<usize>;

    fn try_get_raw(&self) -> crate::Result<&[u8]>;
}

// TODO: fn type_info_of(index)
// TODO: fn try_type_info_of(index)
// TODO: trait ColumnIndex
