use crate::decode::Decode;
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

impl<'de> Decode<'de, Postgres> for Vec<u8> {
    fn decode(buf: &'de [u8]) -> crate::Result<Self> {
        Ok(buf.to_vec())
    }
}

impl<'de> Decode<'de, Postgres> for &'de [u8] {
    fn decode(buf: &'de [u8]) -> crate::Result<&'de [u8]> {
        Ok(buf)
    }
}
