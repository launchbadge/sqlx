use std::fmt::{self, Debug, Formatter};

use crate::database::Database;

/// A representation of a statement that _could_ have been executed against the database.
///
/// Returned from [`Executor::describe`](crate::executor::Executor::describe).
///
/// The compile-time verification within the query macros utilizes `describe` and this type to
/// act on an arbitrary query.
#[derive(Debug)]
#[non_exhaustive]
pub struct Describe<DB>
where
    DB: Database,
{
    /// The expected types of the parameters. This is currently always an array of `None` values
    /// on all databases drivers aside from PostgreSQL.
    pub parameters: Vec<Option<DB::TypeInfo>>,

    /// The columns that will be found in the results from this query.
    pub columns: Vec<Column<DB>>,
}

#[derive(Debug)]
#[non_exhaustive]
pub struct Column<DB>
where
    DB: Database,
{
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
