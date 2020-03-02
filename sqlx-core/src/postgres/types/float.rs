use std::convert::TryInto;
use std::str::FromStr;

use byteorder::{NetworkEndian, ReadBytesExt};

use crate::decode::Decode;
use crate::encode::Encode;
use crate::error::Error;
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
        match value.try_into()? {
            PgValue::Binary(mut buf) => buf
                .read_i32::<NetworkEndian>()
                .map_err(Error::decode)
                .map(|value| f32::from_bits(value as u32)),

            PgValue::Text(s) => f32::from_str(s).map_err(Error::decode),
        }
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
        match value.try_into()? {
            PgValue::Binary(mut buf) => buf
                .read_i64::<NetworkEndian>()
                .map_err(Error::decode)
                .map(|value| f64::from_bits(value as u64)),

            PgValue::Text(s) => f64::from_str(s).map_err(Error::decode),
        }
    }
}
