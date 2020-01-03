use crate::decode::{Decode, DecodeError};
use crate::encode::Encode;
use crate::postgres::types::PgTypeMetadata;
use crate::types::{HasSqlType, HasTypeMetadata};
use crate::Postgres;
use std::str;

impl HasSqlType<str> for Postgres {
    fn metadata() -> PgTypeMetadata {
        PgTypeMetadata::binary(25, 1009)
    }

    fn compatible_types() -> &'static [Self::TypeId] {
        &[
            18, // char
            19, // name
            25, // text
            1043 // varchar
        ]
    }
}

impl HasSqlType<String> for Postgres {
    fn metadata() -> PgTypeMetadata {
        <Postgres as HasSqlType<str>>::metadata()
    }

    fn compatible_types() -> &'static [Self::TypeId] {
        <Postgres as HasSqlType<str>>::compatible_types()
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
