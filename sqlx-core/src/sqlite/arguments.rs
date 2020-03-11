use crate::arguments::Arguments;
use crate::encode::Encode;
use crate::sqlite::Sqlite;
use crate::types::Type;
use crate::sqlite::value::SqliteArgumentValue;

#[derive(Default)]
pub struct SqliteArguments {
    values: Vec<SqliteArgumentValue>,
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
