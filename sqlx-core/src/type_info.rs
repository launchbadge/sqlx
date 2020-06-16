use std::fmt::{Debug, Display};

/// Provides information about a SQL type for the database driver.
///
/// Currently this only exposes type equality rules that should roughly match the interpretation
/// in a given database (e.g., in PostgreSQL `VARCHAR` and `TEXT` are roughly equivalent
/// apart from storage).
pub trait TypeInfo: Debug + Display + Clone + PartialEq<Self> {
    /// Returns the database system name of the type. Length specifiers should not be included.
    /// Common type names are `VARCHAR`, `TEXT`, or `INT`. Type names should be uppercase. They
    /// should be a rough approximation of how they are written in SQL in the given database.
    fn name(&self) -> &str;
}
