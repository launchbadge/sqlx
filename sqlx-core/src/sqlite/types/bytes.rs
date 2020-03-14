use crate::decode::Decode;
use crate::encode::Encode;
use crate::sqlite::types::{SqliteType, SqliteTypeAffinity};
use crate::sqlite::{Sqlite, SqliteArgumentValue, SqliteResultValue, SqliteTypeInfo};
use crate::types::Type;

impl Type<Sqlite> for [u8] {
    fn type_info() -> SqliteTypeInfo {
        SqliteTypeInfo::new(SqliteType::Blob, SqliteTypeAffinity::Blob)
    }
}

impl Type<Sqlite> for Vec<u8> {
    fn type_info() -> SqliteTypeInfo {
        <[u8] as Type<Sqlite>>::type_info()
    }
}

impl Encode<Sqlite> for [u8] {
    fn encode(&self, values: &mut Vec<SqliteArgumentValue>) {
        // TODO: look into a way to remove this allocation
        values.push(SqliteArgumentValue::Blob(self.to_owned()));
    }
}

impl Encode<Sqlite> for Vec<u8> {
    fn encode(&self, values: &mut Vec<SqliteArgumentValue>) {
        <[u8] as Encode<Sqlite>>::encode(self, values)
    }
}

impl<'de> Decode<'de, Sqlite> for &'de [u8] {
    fn decode(value: SqliteResultValue<'de>) -> crate::Result<&'de [u8]> {
        value.blob()
    }
}

impl<'de> Decode<'de, Sqlite> for Vec<u8> {
    fn decode(value: SqliteResultValue<'de>) -> crate::Result<Vec<u8>> {
        <&[u8] as Decode<Sqlite>>::decode(value).map(ToOwned::to_owned)
    }
}
