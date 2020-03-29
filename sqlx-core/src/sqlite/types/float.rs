use crate::decode::Decode;
use crate::encode::Encode;
use crate::sqlite::type_info::{SqliteType, SqliteTypeAffinity};
use crate::sqlite::{Sqlite, SqliteArgumentValue, SqliteTypeInfo, SqliteValue};
use crate::types::Type;

impl Type<Sqlite> for f32 {
    fn type_info() -> SqliteTypeInfo {
        SqliteTypeInfo::new(SqliteType::Float, SqliteTypeAffinity::Real)
    }
}

impl Encode<Sqlite> for f32 {
    fn encode(&self, values: &mut Vec<SqliteArgumentValue>) {
        values.push(SqliteArgumentValue::Double((*self).into()));
    }
}

impl<'a> Decode<'a, Sqlite> for f32 {
    fn decode(value: SqliteValue<'a>) -> crate::Result<f32> {
        Ok(value.double() as f32)
    }
}

impl Type<Sqlite> for f64 {
    fn type_info() -> SqliteTypeInfo {
        SqliteTypeInfo::new(SqliteType::Float, SqliteTypeAffinity::Real)
    }
}

impl Encode<Sqlite> for f64 {
    fn encode(&self, values: &mut Vec<SqliteArgumentValue>) {
        values.push(SqliteArgumentValue::Double((*self).into()));
    }
}

impl<'a> Decode<'a, Sqlite> for f64 {
    fn decode(value: SqliteValue<'a>) -> crate::Result<f64> {
        Ok(value.double())
    }
}
