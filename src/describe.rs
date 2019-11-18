use crate::Backend;

use crate::types::HasTypeMetadata;

/// The result of running prepare + describe for the given backend.
pub struct Describe<DB: Backend> {
    ///
    /// The expected type IDs of bind parameters.
    pub param_types: Vec<<DB as HasTypeMetadata>::TypeId>,
    ///
    pub columns: Vec<Column<DB>>,
}

pub struct Column<DB: Backend> {
    pub name: Option<String>,
    pub table_id: Option<<DB as Backend>::TableIdent>,
    /// The type ID of this result column.
    pub type_id: <DB as HasTypeMetadata>::TypeId,
}
