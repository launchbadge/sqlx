use crate::Database;

/// Represents a column from a query.
pub trait Column {
    type Database: Database;

    /// Returns the name of the column.
    fn name(&self) -> &str;

    /// Returns the (zero-based) position of the column.
    fn ordinal(&self) -> usize;

    /// Returns type information of the column.
    fn type_info(&self) -> &<Self::Database as Database>::TypeInfo;
}
