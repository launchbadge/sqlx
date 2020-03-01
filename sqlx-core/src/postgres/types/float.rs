use crate::decode::Decode;
use crate::encode::Encode;
use crate::postgres::protocol::TypeId;
use crate::postgres::types::PgTypeInfo;
use crate::postgres::{PgValue, Postgres};
use crate::types::Type;

impl Type<Postgres> for f32 {
    fn type_info() -> PgTypeInfo {
        PgTypeInfo::new(TypeId::FLOAT4)
    }
}

impl Type<Postgres> for [f32] {
    fn type_info() -> PgTypeInfo {
        PgTypeInfo::new(TypeId::ARRAY_FLOAT4)
    }
}

impl Encode<Postgres> for f32 {
    fn encode(&self, buf: &mut Vec<u8>) {
        <i32 as Encode<Postgres>>::encode(&(self.to_bits() as i32), buf)
    }
}

impl<'de> Decode<'de, Postgres> for f32 {
    fn decode(value: Option<PgValue<'de>>) -> crate::Result<Self> {
        <i32 as Decode<Postgres>>::decode(value).map(|value| f32::from_bits(value as u32))
    }
}

impl Type<Postgres> for f64 {
    fn type_info() -> PgTypeInfo {
        PgTypeInfo::new(TypeId::FLOAT8)
    }
}

impl Type<Postgres> for [f64] {
    fn type_info() -> PgTypeInfo {
        PgTypeInfo::new(TypeId::ARRAY_FLOAT8)
    }
}

impl Encode<Postgres> for f64 {
    fn encode(&self, buf: &mut Vec<u8>) {
        <i64 as Encode<Postgres>>::encode(&(self.to_bits() as i64), buf)
    }
}

impl<'de> Decode<'de, Postgres> for f64 {
    fn decode(value: Option<PgValue<'de>>) -> crate::Result<Self> {
        <i32 as Decode<Postgres>>::decode(value).map(|value| f64::from_bits(value as u64))
    }
}
