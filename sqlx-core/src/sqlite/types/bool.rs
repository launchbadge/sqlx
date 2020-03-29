use crate::decode::Decode;
use crate::encode::Encode;
use crate::sqlite::type_info::{SqliteType, SqliteTypeAffinity};
use crate::sqlite::{Sqlite, SqliteArgumentValue, SqliteTypeInfo, SqliteValue};
use crate::types::Type;

impl Type<Sqlite> for bool {
    fn type_info() -> SqliteTypeInfo {
        SqliteTypeInfo::new(SqliteType::Boolean, SqliteTypeAffinity::Numeric)
    }
}

impl Encode<Sqlite> for bool {
    fn encode(&self, values: &mut Vec<SqliteArgumentValue>) {
        values.push(SqliteArgumentValue::Int((*self).into()));
    }
}

impl<'a> Decode<'a, Sqlite> for bool {
    fn decode(value: SqliteValue<'a>) -> crate::Result<bool> {
        Ok(value.int() != 0)
    }
}
