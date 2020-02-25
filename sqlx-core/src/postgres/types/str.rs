use std::str;

use crate::decode::{Decode, DecodeError};
use crate::encode::Encode;
use crate::postgres::protocol::TypeId;
use crate::postgres::types::PgTypeInfo;
use crate::types::HasSqlType;
use crate::Postgres;

impl HasSqlType<str> for Postgres {
    fn type_info() -> PgTypeInfo {
        PgTypeInfo::new(TypeId::TEXT)
    }
}

impl HasSqlType<[&'_ str]> for Postgres {
    fn type_info() -> PgTypeInfo {
        PgTypeInfo::new(TypeId::ARRAY_TEXT)
    }
}
impl HasSqlType<Vec<&'_ str>> for Postgres {
    fn type_info() -> PgTypeInfo {
        <Self as HasSqlType<[&'_ str]>>::type_info()
    }
}

// TODO: Do we need [HasSqlType] on String here?
impl HasSqlType<String> for Postgres {
    fn type_info() -> PgTypeInfo {
        <Self as HasSqlType<str>>::type_info()
    }
}
impl HasSqlType<[String]> for Postgres {
    fn type_info() -> PgTypeInfo {
        <Self as HasSqlType<[&'_ str]>>::type_info()
    }
}
impl HasSqlType<Vec<String>> for Postgres {
    fn type_info() -> PgTypeInfo {
        <Self as HasSqlType<Vec<&'_ str>>>::type_info()
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

impl Decode<Postgres> for String {
    fn decode(buf: &[u8]) -> Result<Self, DecodeError> {
        Ok(str::from_utf8(buf)?.to_owned())
    }
}
