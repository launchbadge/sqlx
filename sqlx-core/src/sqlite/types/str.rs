use crate::decode::Decode;
use crate::encode::Encode;
use crate::sqlite::types::{SqliteType, SqliteTypeAffinity};
use crate::sqlite::{Sqlite, SqliteArgumentValue, SqliteResultValue, SqliteTypeInfo};
use crate::types::Type;

impl Type<Sqlite> for str {
    fn type_info() -> SqliteTypeInfo {
        SqliteTypeInfo::new(SqliteType::Text, SqliteTypeAffinity::Text)
    }
}

impl Type<Sqlite> for String {
    fn type_info() -> SqliteTypeInfo {
        SqliteTypeInfo::new(SqliteType::Text, SqliteTypeAffinity::Text)
    }
}

impl Encode<Sqlite> for str {
    fn encode(&self, values: &mut Vec<SqliteArgumentValue>) {
        // TODO: look into a way to remove this allocation
        values.push(SqliteArgumentValue::Text(self.to_owned()));
    }
}

impl<'de> Decode<'de, Sqlite> for &'de str {
    fn decode(value: SqliteResultValue<'de>) -> crate::Result<&'de str> {
        value.text()
    }
}

impl<'de> Decode<'de, Sqlite> for String {
    fn decode(value: SqliteResultValue<'de>) -> crate::Result<String> {
        Ok(value.text()?.to_owned())
    }
}
