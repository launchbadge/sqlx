use std::convert::TryInto;
use std::str::from_utf8;

use crate::decode::Decode;
use crate::encode::Encode;
use crate::postgres::protocol::TypeId;
use crate::postgres::row::PgValue;
use crate::postgres::types::PgTypeInfo;
use crate::postgres::Postgres;
use crate::types::Type;
use crate::Error;

impl Type<Postgres> for str {
    fn type_info() -> PgTypeInfo {
        PgTypeInfo::new(TypeId::TEXT, "TEXT")
    }
}

impl Type<Postgres> for [&'_ str] {
    fn type_info() -> PgTypeInfo {
        PgTypeInfo::new(TypeId::ARRAY_TEXT, "TEXT[]")
    }
}

impl Type<Postgres> for Vec<&'_ str> {
    fn type_info() -> PgTypeInfo {
        <[&'_ str] as Type<Postgres>>::type_info()
    }
}

impl Type<Postgres> for String {
    fn type_info() -> PgTypeInfo {
        <str as Type<Postgres>>::type_info()
    }
}

impl Type<Postgres> for [String] {
    fn type_info() -> PgTypeInfo {
        <[&'_ str] as Type<Postgres>>::type_info()
    }
}

impl Type<Postgres> for Vec<String> {
    fn type_info() -> PgTypeInfo {
        <Vec<&'_ str> as Type<Postgres>>::type_info()
    }
}

impl Encode<Postgres> for str {
    fn encode(&self, buf: &mut Vec<u8>) {
        buf.extend_from_slice(self.as_bytes());
    }

    fn size_hint(&self) -> usize {
        self.len()
    }
}

impl Encode<Postgres> for String {
    fn encode(&self, buf: &mut Vec<u8>) {
        <str as Encode<Postgres>>::encode(self.as_str(), buf)
    }

    fn size_hint(&self) -> usize {
        self.len()
    }
}

impl<'de> Decode<'de, Postgres> for String {
    fn decode(buf: Option<PgValue<'de>>) -> crate::Result<Postgres, Self> {
        <&'de str as Decode<Postgres>>::decode(buf).map(ToOwned::to_owned)
    }
}

impl<'de> Decode<'de, Postgres> for &'de str {
    fn decode(value: Option<PgValue<'de>>) -> crate::Result<Postgres, Self> {
        match value.try_into()? {
            PgValue::Binary(buf) => from_utf8(buf).map_err(Error::decode),
            PgValue::Text(s) => Ok(s),
        }
    }
}
