//! Types for returning SQL type information about queries.
//!
//! The compile-time type checking within the query macros heavily lean on the information
//! provided within these types.

use crate::database::Database;

// TODO(@mehcode): Remove [pub] from Describe/Column and use methods to expose the properties

/// A representation of a statement that _could_ have been executed against the database.
///
/// Returned from [`Executor::describe`](crate::executor::Executor::describe).
///
/// The compile-time verification within the query macros utilizes `describe` and this type to
/// act on an arbitrary query.
#[derive(Debug)]
#[non_exhaustive]
#[cfg_attr(feature = "offline", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(
    feature = "offline",
    serde(bound(
        serialize = "DB::TypeInfo: serde::Serialize",
        deserialize = "DB::TypeInfo: serde::de::DeserializeOwned"
    ))
)]
pub struct Describe<DB>
where
    DB: Database,
{
    /// The expected types of the parameters. This is currently always an array of `None` values
    /// on all databases drivers aside from PostgreSQL.
    pub params: Vec<Option<DB::TypeInfo>>,

    /// The columns that will be found in the results from this query.
    pub columns: Vec<Column<DB>>,
}

#[derive(Debug)]
#[non_exhaustive]
#[cfg_attr(feature = "offline", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(
    feature = "offline",
    serde(bound(
        serialize = "DB::TypeInfo: serde::Serialize",
        deserialize = "DB::TypeInfo: serde::de::DeserializeOwned"
    ))
)]
pub struct Column<DB>
where
    DB: Database,
{
    /// The name of the result column.
    ///
    /// The column name is unreliable (and can change between database minor versions) if this
    /// result column is an expression that has not been aliased.
    pub name: String,

    /// The type information for the result column.
    ///
    /// This may be `None` if the type cannot be determined. This occurs in SQLite when
    /// the column is an expression.
    pub type_info: Option<DB::TypeInfo>,

    /// Whether the column cannot be `NULL` (or if that is even knowable).
    /// This value is only not `None` if received from a call to `describe`.
    pub not_null: Option<bool>,
}
