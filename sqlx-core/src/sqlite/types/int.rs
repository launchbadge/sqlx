use crate::decode::Decode;
use crate::encode::Encode;
use crate::sqlite::value::{SqliteArgumentValue, SqliteResultValue};
use crate::sqlite::{Sqlite, SqliteTypeInfo};
use crate::types::Type;

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
