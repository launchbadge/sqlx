use crate::protocol::PrepareOk;
use crate::{MySqlColumn, MySqlTypeInfo};

#[derive(Debug)]
pub(crate) struct RawStatement {
    id: u32,
    pub(crate) columns: Vec<MySqlColumn>,
    pub(crate) parameters: Vec<MySqlTypeInfo>,
}

impl RawStatement {
    pub(crate) fn new(ok: PrepareOk) -> Self {
        Self {
            id: ok.statement_id,
            columns: Vec::with_capacity(ok.columns.into()),
            parameters: Vec::with_capacity(ok.parameters.into()),
        }
    }

    pub(crate) fn id(&self) -> u32 {
        self.id
    }
}
