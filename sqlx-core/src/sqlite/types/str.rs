use crate::decode::Decode;
use crate::encode::Encode;
use crate::error::UnexpectedNullError;
use crate::sqlite::types::{SqliteType, SqliteTypeAffinity};
use crate::sqlite::{Sqlite, SqliteArgumentValue, SqliteTypeInfo, SqliteValue};
use crate::types::Type;

impl Type<Sqlite> for str {
    fn type_info() -> SqliteTypeInfo {
        SqliteTypeInfo::new(SqliteType::Text, SqliteTypeAffinity::Text)
    }
}

impl Type<Sqlite> for String {
    fn type_info() -> SqliteTypeInfo {
        <str as Type<Sqlite>>::type_info()
    }
}

impl Encode<Sqlite> for str {
    fn encode(&self, values: &mut Vec<SqliteArgumentValue>) {
        // TODO: look into a way to remove this allocation
        values.push(SqliteArgumentValue::Text(self.to_owned()));
    }
}

impl Encode<Sqlite> for String {
    fn encode(&self, values: &mut Vec<SqliteArgumentValue>) {
        <str as Encode<Sqlite>>::encode(self, values)
    }
}

impl<'de> Decode<'de, Sqlite> for &'de str {
    fn decode(value: SqliteValue<'de>) -> crate::Result<Sqlite, &'de str> {
        value
            .text()
            .ok_or_else(|| crate::Error::decode(UnexpectedNullError))
    }
}

impl<'de> Decode<'de, Sqlite> for String {
    fn decode(value: SqliteValue<'de>) -> crate::Result<Sqlite, String> {
        <&str as Decode<Sqlite>>::decode(value).map(ToOwned::to_owned)
    }
}
