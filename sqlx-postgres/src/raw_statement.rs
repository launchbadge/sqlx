use crate::protocol::frontend::StatementId;
use crate::{PgColumn, PgTypeInfo};

#[derive(Debug, Clone)]
pub(crate) struct RawStatement {
    pub(crate) id: StatementId,
    pub(crate) columns: Vec<PgColumn>,
    pub(crate) parameters: Vec<PgTypeInfo>,
}

impl RawStatement {
    pub(crate) fn new(id: StatementId) -> Self {
        Self { id, columns: Vec::new(), parameters: Vec::new() }
    }
}
