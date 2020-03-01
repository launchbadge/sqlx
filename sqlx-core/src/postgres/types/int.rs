use byteorder::{ByteOrder, NetworkEndian};

use crate::decode::Decode;
use crate::encode::Encode;
use crate::postgres::protocol::TypeId;
use crate::postgres::types::PgTypeInfo;
use crate::postgres::Postgres;
use crate::types::Type;

impl Type<Postgres> for i16 {
    fn type_info() -> PgTypeInfo {
        PgTypeInfo::new(TypeId::INT2)
    }
}

impl Type<Postgres> for [i16] {
    fn type_info() -> PgTypeInfo {
        PgTypeInfo::new(TypeId::ARRAY_INT2)
    }
}

impl Encode<Postgres> for i16 {
    fn encode(&self, buf: &mut Vec<u8>) {
        buf.extend_from_slice(&self.to_be_bytes());
    }
}

impl<'de> Decode<'de, Postgres> for i16 {
    fn decode(buf: &'de [u8]) -> crate::Result<Self> {
        Ok(NetworkEndian::read_i16(buf))
    }
}

impl Type<Postgres> for i32 {
    fn type_info() -> PgTypeInfo {
        PgTypeInfo::new(TypeId::INT4)
    }
}

impl Type<Postgres> for [i32] {
    fn type_info() -> PgTypeInfo {
        PgTypeInfo::new(TypeId::ARRAY_INT4)
    }
}

impl Encode<Postgres> for i32 {
    fn encode(&self, buf: &mut Vec<u8>) {
        buf.extend_from_slice(&self.to_be_bytes());
    }
}

impl<'de> Decode<'de, Postgres> for i32 {
    fn decode(buf: &'de [u8]) -> crate::Result<Self> {
        Ok(NetworkEndian::read_i32(buf))
    }
}

impl Type<Postgres> for i64 {
    fn type_info() -> PgTypeInfo {
        PgTypeInfo::new(TypeId::INT8)
    }
}

impl Type<Postgres> for [i64] {
    fn type_info() -> PgTypeInfo {
        PgTypeInfo::new(TypeId::ARRAY_INT8)
    }
}

impl Encode<Postgres> for i64 {
    fn encode(&self, buf: &mut Vec<u8>) {
        buf.extend_from_slice(&self.to_be_bytes());
    }
}

impl<'de> Decode<'de, Postgres> for i64 {
    fn decode(buf: &'de [u8]) -> crate::Result<Self> {
        Ok(NetworkEndian::read_i64(buf))
    }
}
