use crate::database::Database;
use std::fmt::Debug;

pub trait Column: private_column::Sealed + 'static + Send + Sync + Debug {
    type Database: Database;

    /// Gets the column ordinal.
    ///
    /// This can be used to unambiguously refer to this column within a row in case more than
    /// one column have the same name
    fn ordinal(&self) -> usize;

    /// Gets the column name or alias.
    ///
    /// The column name is unreliable (and can change between database minor versions) if this
    /// column is an expression that has not been aliased.
    fn name(&self) -> &str;

    /// Gets the type information for the column.
    fn type_info(&self) -> &<Self::Database as Database>::TypeInfo;
}

// Prevent users from implementing the `Row` trait.
pub(crate) mod private_column {
    pub trait Sealed {}
}
