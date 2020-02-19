use crate::decode::{Decode, DecodeError};
use crate::encode::Encode;
use crate::postgres::protocol::TypeId;
use crate::postgres::types::PgTypeInfo;
use crate::postgres::Postgres;
use crate::types::Type;

impl Type<Postgres> for [u8] {
    fn type_info() -> PgTypeInfo {
        PgTypeInfo::new(TypeId::BYTEA)
    }
}

impl Type<Postgres> for [&'_ [u8]] {
    fn type_info() -> PgTypeInfo {
        PgTypeInfo::new(TypeId::ARRAY_BYTEA)
    }
}

// TODO: Do we need the [HasSqlType] here on the Vec?
impl Type<Postgres> for Vec<u8> {
    fn type_info() -> PgTypeInfo {
        <[u8] as Type<Postgres>>::type_info()
    }
}

impl Encode<Postgres> for [u8] {
    fn encode(&self, buf: &mut Vec<u8>) {
        buf.extend_from_slice(self);
    }
}

impl Encode<Postgres> for Vec<u8> {
    fn encode(&self, buf: &mut Vec<u8>) {
        <[u8] as Encode<Postgres>>::encode(self, buf);
    }
}

impl Decode<Postgres> for Vec<u8> {
    fn decode(buf: &[u8]) -> Result<Self, DecodeError> {
        Ok(buf.to_vec())
    }
}
