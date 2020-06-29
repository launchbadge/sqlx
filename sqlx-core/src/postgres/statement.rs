use super::{PgTypeInfo, Postgres};
use crate::describe::Column;

#[derive(Debug)]
pub struct Statement {
    id: u32,
    params: Vec<Option<PgTypeInfo>>,
    columns: Vec<Column<Postgres>>,
    columns_set: bool,
}

impl Statement {
    pub fn new(id: u32, params: Vec<Option<PgTypeInfo>>) -> Self {
        Self {
            id,
            params,
            columns: Vec::new(),
            columns_set: false,
        }
    }

    pub fn id(&self) -> u32 {
        self.id
    }

    pub fn params(&self) -> &[Option<PgTypeInfo>] {
        self.params.as_slice()
    }

    pub fn columns(&self) -> &[Column<Postgres>] {
        self.columns.as_slice()
    }

    pub fn set_columns(&mut self, columns: Vec<Column<Postgres>>) {
        self.columns = columns;
        self.columns_set = true;
    }

    pub fn columns_set(&self) -> bool {
        self.columns_set
    }
}
