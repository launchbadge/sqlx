use crate::types::Type;
use crate::sqlite::{Sqlite, SqliteTypeInfo};
use crate::encode::Encode;
use crate::sqlite::value::{SqliteArgumentValue, SqliteResultValue};
use crate::decode::Decode;

impl Type<Sqlite> for i32 {
    fn type_info() -> SqliteTypeInfo {
        // SqliteTypeInfo::new(ValueKind::Int)
        todo!()
    }
}

impl Encode<Sqlite> for i32 {
    fn encode(&self, values: &mut Vec<SqliteArgumentValue>) {
        values.push(SqliteArgumentValue::Int((*self).into()));
    }
}

impl<'a> Decode<'a, Sqlite> for i32 {
    fn decode(value: SqliteResultValue<'a>) -> crate::Result<i32> {
        // Ok(value.int())
        todo!()
    }
}
