use crate::decode::Decode;
use crate::encode::Encode;
use crate::sqlite::types::{SqliteType, SqliteTypeAffinity};
use crate::sqlite::{Sqlite, SqliteArgumentValue, SqliteResultValue, SqliteTypeInfo};
use crate::types::Type;

impl Type<Sqlite> for bool {
    fn type_info() -> SqliteTypeInfo {
        SqliteTypeInfo::new(SqliteType::Integer, SqliteTypeAffinity::Numeric)
    }
}

impl Encode<Sqlite> for bool {
    fn encode(&self, values: &mut Vec<SqliteArgumentValue>) {
        values.push(SqliteArgumentValue::Int((*self).into()));
    }
}

impl<'a> Decode<'a, Sqlite> for bool {
    fn decode(value: SqliteResultValue<'a>) -> crate::Result<bool> {
        Ok(value.int() != 0)
    }
}
