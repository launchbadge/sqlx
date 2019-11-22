use crate::Backend;

use crate::types::HasTypeMetadata;

use std::fmt;

/// The result of running prepare + describe for the given backend.
pub struct Describe<DB: Backend> {
    /// The expected type IDs of bind parameters.
    pub param_types: Vec<<DB as HasTypeMetadata>::TypeId>,
    ///
    pub result_fields: Vec<ResultField<DB>>,
}

impl<DB: Backend> fmt::Debug for Describe<DB>
where
    <DB as HasTypeMetadata>::TypeId: fmt::Debug,
    ResultField<DB>: fmt::Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("Describe")
            .field("param_types", &self.param_types)
            .field("result_fields", &self.result_fields)
            .finish()
    }
}

pub struct ResultField<DB: Backend> {
    pub name: Option<String>,
    pub table_id: Option<<DB as Backend>::TableIdent>,
    /// The type ID of this result column.
    pub type_id: <DB as HasTypeMetadata>::TypeId,
}

impl<DB: Backend> fmt::Debug for ResultField<DB>
where
    <DB as Backend>::TableIdent: fmt::Debug,
    <DB as HasTypeMetadata>::TypeId: fmt::Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("ResultField")
            .field("name", &self.name)
            .field("table_id", &self.table_id)
            .field("type_id", &self.type_id)
            .finish()
    }
}
