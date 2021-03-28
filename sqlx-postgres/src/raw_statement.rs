use crate::{PgColumn, PgTypeInfo};

#[derive(Debug, Clone)]
pub(crate) struct RawStatement {
    pub(crate) id: u32,
    pub(crate) columns: Vec<PgColumn>,
    pub(crate) parameters: Vec<PgTypeInfo>,
}

impl RawStatement {
    pub(crate) fn new(id: u32) -> Self {
        Self { id, columns: Vec::new(), parameters: Vec::new() }
    }
}
