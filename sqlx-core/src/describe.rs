//! Types for returning SQL type information about queries.

use crate::types::HasTypeMetadata;
use crate::Database;
use std::fmt::{self, Debug};

/// The return type of [Executor::describe].
#[non_exhaustive]
pub struct Describe<DB>
where
    DB: Database + ?Sized,
{
    /// The expected types for the parameters of the query.
    pub param_types: Box<[<DB as HasTypeMetadata>::TypeId]>,

    /// The type and table information, if any for the results of the query.
    pub result_columns: Box<[Column<DB>]>,
}

impl<DB> Debug for Describe<DB>
where
    DB: Database,
    <DB as HasTypeMetadata>::TypeId: Debug,
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
#[non_exhaustive]
pub struct Column<DB>
where
    DB: Database + ?Sized,
{
    pub name: Option<Box<str>>,
    pub table_id: Option<<DB as HasTypeMetadata>::TableId>,
    pub type_id: <DB as HasTypeMetadata>::TypeId,
}

impl<DB> Debug for Column<DB>
where
    DB: Database + ?Sized,
    <DB as HasTypeMetadata>::TableId: Debug,
    <DB as HasTypeMetadata>::TypeId: Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("Column")
            .field("name", &self.name)
            .field("table_id", &self.table_id)
            .field("type_id", &self.type_id)
            .finish()
    }
}
