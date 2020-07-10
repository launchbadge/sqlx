use super::{PgColumn, PgTypeInfo};

#[derive(Debug, Clone)]
pub struct PgStatement {
    pub(crate) id: u32,
    pub(crate) columns: Vec<PgColumn>,
    pub(crate) parameters: Vec<PgTypeInfo>,
}
