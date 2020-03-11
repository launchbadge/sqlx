#[derive(Debug, Clone)]
pub enum SqliteArgumentValue {
    // TODO: Take by reference to remove the allocation
    Text(String),

    // TODO: Take by reference to remove the allocation
    Blob(Vec<u8>),

    Double(f64),

    Int(i64),
}

pub struct SqliteResultValue<'c> {
    // statement: SqliteStatement<'c>,
    statement: std::marker::PhantomData<&'c ()>,
}
