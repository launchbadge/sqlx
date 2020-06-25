use std::fmt::{Debug, Display};

/// Provides information about a SQL type for the database driver.
pub trait TypeInfo: Debug + Display + Clone + PartialEq<Self> {
    /// Returns the database system name of the type. Length specifiers should not be included.
    /// Common type names are `VARCHAR`, `TEXT`, or `INT`. Type names should be uppercase. They
    /// should be a rough approximation of how they are written in SQL in the given database.
    fn name(&self) -> &str;
}
