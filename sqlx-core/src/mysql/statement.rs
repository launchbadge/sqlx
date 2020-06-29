use super::{MySql, MySqlTypeInfo};
use crate::describe::Column;

#[derive(Debug)]
pub struct Statement {
    pub(crate) id: u32,
    pub(crate) params: Vec<Option<MySqlTypeInfo>>,
    pub(crate) columns: Vec<Column<MySql>>,
}
