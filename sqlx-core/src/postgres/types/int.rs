use byteorder::{ByteOrder, NetworkEndian};

use crate::decode::{Decode, DecodeError};
use crate::encode::Encode;
use crate::postgres::protocol::TypeId;
use crate::postgres::types::PgTypeInfo;
use crate::postgres::Postgres;
use crate::types::HasSqlType;

impl HasSqlType<i16> for Postgres {
    fn type_info() -> PgTypeInfo {
        PgTypeInfo::new(TypeId::INT2)
    }
}

impl HasSqlType<[i16]> for Postgres {
    fn type_info() -> PgTypeInfo {
        PgTypeInfo::new(TypeId::ARRAY_INT2)
    }
}
impl HasSqlType<Vec<i16>> for Postgres {
    fn type_info() -> PgTypeInfo {
        <Self as HasSqlType<[i16]>>::type_info()
    }
}

impl Encode<Postgres> for i16 {
    fn encode(&self, buf: &mut Vec<u8>) {
        buf.extend_from_slice(&self.to_be_bytes());
    }
}

impl Decode<Postgres> for i16 {
    fn decode(buf: &[u8]) -> Result<Self, DecodeError> {
        Ok(NetworkEndian::read_i16(buf))
    }
}

impl HasSqlType<i32> for Postgres {
    fn type_info() -> PgTypeInfo {
        PgTypeInfo::new(TypeId::INT4)
    }
}

impl HasSqlType<[i32]> for Postgres {
    fn type_info() -> PgTypeInfo {
        PgTypeInfo::new(TypeId::ARRAY_INT4)
    }
}
impl HasSqlType<Vec<i32>> for Postgres {
    fn type_info() -> PgTypeInfo {
        <Self as HasSqlType<[i32]>>::type_info()
    }
}

impl Encode<Postgres> for i32 {
    fn encode(&self, buf: &mut Vec<u8>) {
        buf.extend_from_slice(&self.to_be_bytes());
    }
}

impl Decode<Postgres> for i32 {
    fn decode(buf: &[u8]) -> Result<Self, DecodeError> {
        Ok(NetworkEndian::read_i32(buf))
    }
}

impl HasSqlType<i64> for Postgres {
    fn type_info() -> PgTypeInfo {
        PgTypeInfo::new(TypeId::INT8)
    }
}

impl HasSqlType<[i64]> for Postgres {
    fn type_info() -> PgTypeInfo {
        PgTypeInfo::new(TypeId::ARRAY_INT8)
    }
}
impl HasSqlType<Vec<i64>> for Postgres {
    fn type_info() -> PgTypeInfo {
        <Self as HasSqlType<[i64]>>::type_info()
    }
}

impl Encode<Postgres> for i64 {
    fn encode(&self, buf: &mut Vec<u8>) {
        buf.extend_from_slice(&self.to_be_bytes());
    }
}

impl Decode<Postgres> for i64 {
    fn decode(buf: &[u8]) -> Result<Self, DecodeError> {
        Ok(NetworkEndian::read_i64(buf))
    }
}
