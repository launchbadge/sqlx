use super::MySqlColumn;

#[derive(Debug)]
pub struct MySqlStatement {
    pub(crate) id: u32,
    pub(crate) columns: Vec<MySqlColumn>,
    pub(crate) parameters: usize,
    pub(crate) nullable: Vec<Option<bool>>,
}
