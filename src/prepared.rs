#[derive(Debug)]
pub struct PreparedStatement {
    pub name: String,
    pub param_types: Box<[u32]>,
    pub fields: Vec<Field>,
}

#[derive(Debug)]
pub struct Field {
    pub name: String,
    pub table_id: u32,
    pub type_id: u32,
}
