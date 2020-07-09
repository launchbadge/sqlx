use super::MySqlColumn;

pub struct MySqlStatement {
    pub(crate) id: u32,
    pub(crate) columns: Vec<MySqlColumn>,
    pub(crate) parameters: usize,
    pub(crate) nullable: Vec<Option<bool>>,
}
