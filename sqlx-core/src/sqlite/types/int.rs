use crate::decode::Decode;
use crate::encode::Encode;
use crate::sqlite::types::{SqliteType, SqliteTypeAffinity};
use crate::sqlite::{Sqlite, SqliteArgumentValue, SqliteResultValue, SqliteTypeInfo};
use crate::types::Type;

impl Type<Sqlite> for i32 {
    fn type_info() -> SqliteTypeInfo {
        SqliteTypeInfo::new(SqliteType::Integer, SqliteTypeAffinity::Integer)
    }
}

impl Encode<Sqlite> for i32 {
    fn encode(&self, values: &mut Vec<SqliteArgumentValue>) {
        values.push(SqliteArgumentValue::Int((*self).into()));
    }
}

impl<'a> Decode<'a, Sqlite> for i32 {
    fn decode(value: SqliteResultValue<'a>) -> crate::Result<i32> {
        Ok(value.int())
    }
}

impl Type<Sqlite> for i64 {
    fn type_info() -> SqliteTypeInfo {
        SqliteTypeInfo::new(SqliteType::Integer, SqliteTypeAffinity::Integer)
    }
}

impl Encode<Sqlite> for i64 {
    fn encode(&self, values: &mut Vec<SqliteArgumentValue>) {
        values.push(SqliteArgumentValue::Int64((*self).into()));
    }
}

impl<'a> Decode<'a, Sqlite> for i64 {
    fn decode(value: SqliteResultValue<'a>) -> crate::Result<i64> {
        Ok(value.int64())
    }
}
