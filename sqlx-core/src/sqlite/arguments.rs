use crate::arguments::Arguments;
use crate::encode::Encode;
use crate::sqlite::Sqlite;
use crate::types::Type;

#[derive(Debug, Clone)]
pub enum SqliteValue {
    // TODO: Take by reference to remove the allocation
    Text(String),

    // TODO: Take by reference to remove the allocation
    Blob(Vec<u8>),

    Double(f64),

    Int(i64),
}

#[derive(Default)]
pub struct SqliteArguments {
    values: Vec<SqliteValue>,
}

impl Arguments for SqliteArguments {
    type Database = Sqlite;

    fn reserve(&mut self, len: usize, _size_hint: usize) {
        self.values.reserve(1);
    }

    fn add<T>(&mut self, value: T)
    where
        T: Encode<Self::Database> + Type<Self::Database>,
    {
        value.encode(&mut self.values);
    }
}
