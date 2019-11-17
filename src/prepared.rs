use crate::Backend;

use crate::types::HasTypeMetadata;

/// A prepared statement.
pub struct PreparedStatement<DB: Backend> {
    ///
    pub identifier: <DB as Backend>::StatementIdent,
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
