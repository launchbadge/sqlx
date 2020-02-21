use crate::decode::{Decode, DecodeError};
use crate::encode::Encode;
use crate::postgres::protocol::TypeId;
use crate::postgres::types::PgTypeInfo;
use crate::postgres::Postgres;
use crate::types::HasSqlType;

impl HasSqlType<f32> for Postgres {
    fn type_info() -> PgTypeInfo {
        PgTypeInfo::new(TypeId::FLOAT4)
    }
}

impl HasSqlType<[f32]> for Postgres {
    fn type_info() -> PgTypeInfo {
        PgTypeInfo::new(TypeId::ARRAY_FLOAT4)
    }
}
impl HasSqlType<Vec<f32>> for Postgres {
    fn type_info() -> PgTypeInfo {
        <Self as HasSqlType<[f32]>>::type_info()
    }
}

impl Encode<Postgres> for f32 {
    fn encode(&self, buf: &mut Vec<u8>) {
        <i32 as Encode<Postgres>>::encode(&(self.to_bits() as i32), buf)
    }
}

impl Decode<Postgres> for f32 {
    fn decode(buf: &[u8]) -> Result<Self, DecodeError> {
        Ok(f32::from_bits(
            <i32 as Decode<Postgres>>::decode(buf)? as u32
        ))
    }
}

impl HasSqlType<f64> for Postgres {
    fn type_info() -> PgTypeInfo {
        PgTypeInfo::new(TypeId::FLOAT8)
    }
}

impl HasSqlType<[f64]> for Postgres {
    fn type_info() -> PgTypeInfo {
        PgTypeInfo::new(TypeId::ARRAY_FLOAT8)
    }
}
impl HasSqlType<Vec<f64>> for Postgres {
    fn type_info() -> PgTypeInfo {
        <Self as HasSqlType<[f64]>>::type_info()
    }
}

impl Encode<Postgres> for f64 {
    fn encode(&self, buf: &mut Vec<u8>) {
        <i64 as Encode<Postgres>>::encode(&(self.to_bits() as i64), buf)
    }
}

impl Decode<Postgres> for f64 {
    fn decode(buf: &[u8]) -> Result<Self, DecodeError> {
        Ok(f64::from_bits(
            <i64 as Decode<Postgres>>::decode(buf)? as u64
        ))
    }
}
