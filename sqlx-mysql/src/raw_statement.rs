use sqlx_core::Describe;

use crate::protocol::PrepareOk;
use crate::{MySql, MySqlColumn, MySqlTypeInfo};

#[derive(Debug, Clone)]
pub(crate) struct RawStatement {
    id: u32,
    columns: Vec<MySqlColumn>,
    parameters: Vec<MySqlTypeInfo>,
}

impl RawStatement {
    pub(crate) fn new(ok: &PrepareOk) -> Self {
        Self {
            id: ok.statement_id,
            columns: Vec::with_capacity(ok.columns.into()),
            parameters: Vec::with_capacity(ok.params.into()),
        }
    }

    pub(crate) const fn id(&self) -> u32 {
        self.id
    }

    pub(crate) fn columns(&self) -> &[MySqlColumn] {
        &self.columns
    }

    pub(crate) fn columns_mut(&mut self) -> &mut Vec<MySqlColumn> {
        &mut self.columns
    }

    pub(crate) fn parameters(&self) -> &[MySqlTypeInfo] {
        &self.parameters
    }

    pub(crate) fn parameters_mut(&mut self) -> &mut Vec<MySqlTypeInfo> {
        &mut self.parameters
    }
}

impl From<RawStatement> for Describe<MySql> {
    fn from(stmt: RawStatement) -> Self {
        Self {
            nullable: stmt.columns.iter().map(|col| Some(col.is_nullable())).collect(),
            columns: stmt.columns,
            parameters: stmt.parameters,
        }
    }
}
