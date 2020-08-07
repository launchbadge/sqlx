use crate::database::{Database, HasTypeId};

/// Provides information about a SQL type for the database driver.
pub trait TypeInfo {
    /// The database this type information is for.
    type Database: Database;

    /// Returns the globally-unique identifier for this SQL type.
    fn id(&self) -> <Self::Database as HasTypeId<'_>>::TypeId;

    /// Returns `true` if the database could not determine the actual type.
    ///
    /// Most commonly this occurs in cases where `NULL` is directly used in
    /// an expression.
    ///
    fn is_unknown(&self) -> bool;

    /// Returns `true` if this is a zero-sized type intended to never hold
    /// a value, such as `void` in C.
    ///
    /// PostgreSQL can return this type for simple function expressions
    /// where the function has no return type.
    ///
    fn is_void(&self) -> bool;

    /// Returns the name of this SQL type for this database.
    ///
    /// Length specifiers will not be included. Only the basename will
    /// be returned. This should be a rough approximation of how they are
    /// written in SQL in the given database.
    ///
    /// Common type names include `VARCHAR`, `INTEGER`, and `BIGINT`.
    ///
    fn name(&self) -> &str;
}
