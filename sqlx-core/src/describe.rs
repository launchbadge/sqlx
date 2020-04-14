//! Types for returning SQL type information about queries.

use std::fmt::{self, Debug};

use crate::database::Database;

/// The return type of [`Executor::describe`].
///
/// [`Executor::describe`]: crate::executor::Executor::describe
#[cfg_attr(feature = "offline", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(
    feature = "offline",
    serde(bound(
        serialize = "DB::TypeInfo: serde::Serialize, Column<DB>: serde::Serialize",
        deserialize = "DB::TypeInfo: serde::de::DeserializeOwned, Column<DB>: serde::de::DeserializeOwned"
    ))
)]
#[non_exhaustive]
pub struct Describe<DB>
where
    DB: Database + ?Sized,
{
    // TODO: Describe#param_types should probably be Option<TypeInfo[]> as we either know all the params or we know none
    /// The expected types for the parameters of the query.
    pub param_types: Box<[Option<DB::TypeInfo>]>,

    /// The type and table information, if any for the results of the query.
    pub result_columns: Box<[Column<DB>]>,
}

impl<DB> Debug for Describe<DB>
where
    DB: Database,
    DB::TypeInfo: Debug,
    Column<DB>: Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("Describe")
            .field("param_types", &self.param_types)
            .field("result_columns", &self.result_columns)
            .finish()
    }
}

/// A single column of a result set.
#[cfg_attr(feature = "offline", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(
    feature = "offline",
    serde(bound(
        serialize = "DB::TableId: serde::Serialize, DB::TypeInfo: serde::Serialize",
        deserialize = "DB::TableId: serde::de::DeserializeOwned, DB::TypeInfo: serde::de::DeserializeOwned"
    ))
)]
#[non_exhaustive]
pub struct Column<DB>
where
    DB: Database + ?Sized,
{
    pub name: Option<Box<str>>,
    pub table_id: Option<DB::TableId>,
    pub type_info: Option<DB::TypeInfo>,
    /// Whether or not the column cannot be `NULL` (or if that is even knowable).
    pub non_null: Option<bool>,
}

impl<DB> Debug for Column<DB>
where
    DB: Database + ?Sized,
    DB::TableId: Debug,
    DB::TypeInfo: Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("Column")
            .field("name", &self.name)
            .field("table_id", &self.table_id)
            .field("type_info", &self.type_info)
            .field("non_null", &self.non_null)
            .finish()
    }
}
