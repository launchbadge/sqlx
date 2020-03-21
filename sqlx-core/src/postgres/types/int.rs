use std::convert::TryInto;
use std::str::FromStr;

use byteorder::{NetworkEndian, ReadBytesExt};

use crate::decode::Decode;
use crate::encode::Encode;
use crate::postgres::protocol::TypeId;
use crate::postgres::types::PgTypeInfo;
use crate::postgres::{PgValue, Postgres};
use crate::types::Type;
use crate::Error;

impl Type<Postgres> for i16 {
    fn type_info() -> PgTypeInfo {
        PgTypeInfo::new(TypeId::INT2, "INT2")
    }
}

impl Type<Postgres> for [i16] {
    fn type_info() -> PgTypeInfo {
        PgTypeInfo::new(TypeId::ARRAY_INT2, "INT2[]")
    }
}
impl Type<Postgres> for Vec<i16> {
    fn type_info() -> PgTypeInfo {
        <[i16] as Type<Postgres>>::type_info()
    }
}

impl Encode<Postgres> for i16 {
    fn encode(&self, buf: &mut Vec<u8>) {
        buf.extend_from_slice(&self.to_be_bytes());
    }
}

impl<'de> Decode<'de, Postgres> for i16 {
    fn decode(value: Option<PgValue<'de>>) -> crate::Result<Self> {
        match value.try_into()? {
            PgValue::Binary(mut buf) => buf.read_i16::<NetworkEndian>().map_err(Error::decode),
            PgValue::Text(s) => i16::from_str(s).map_err(Error::decode),
        }
    }
}

impl Type<Postgres> for i32 {
    fn type_info() -> PgTypeInfo {
        PgTypeInfo::new(TypeId::INT4, "INT4")
    }
}

impl Type<Postgres> for [i32] {
    fn type_info() -> PgTypeInfo {
        PgTypeInfo::new(TypeId::ARRAY_INT4, "INT4[]")
    }
}
impl Type<Postgres> for Vec<i32> {
    fn type_info() -> PgTypeInfo {
        <[i32] as Type<Postgres>>::type_info()
    }
}

impl Encode<Postgres> for i32 {
    fn encode(&self, buf: &mut Vec<u8>) {
        buf.extend_from_slice(&self.to_be_bytes());
    }
}

impl<'de> Decode<'de, Postgres> for i32 {
    fn decode(value: Option<PgValue<'de>>) -> crate::Result<Self> {
        match value.try_into()? {
            PgValue::Binary(mut buf) => buf.read_i32::<NetworkEndian>().map_err(Error::decode),
            PgValue::Text(s) => i32::from_str(s).map_err(Error::decode),
        }
    }
}

impl Type<Postgres> for i64 {
    fn type_info() -> PgTypeInfo {
        PgTypeInfo::new(TypeId::INT8, "INT8")
    }
}

impl Type<Postgres> for [i64] {
    fn type_info() -> PgTypeInfo {
        PgTypeInfo::new(TypeId::ARRAY_INT8, "INT8[]")
    }
}
impl Type<Postgres> for Vec<i64> {
    fn type_info() -> PgTypeInfo {
        <[i64] as Type<Postgres>>::type_info()
    }
}

impl Encode<Postgres> for i64 {
    fn encode(&self, buf: &mut Vec<u8>) {
        buf.extend_from_slice(&self.to_be_bytes());
    }
}

impl<'de> Decode<'de, Postgres> for i64 {
    fn decode(value: Option<PgValue<'de>>) -> crate::Result<Self> {
        match value.try_into()? {
            PgValue::Binary(mut buf) => buf.read_i64::<NetworkEndian>().map_err(Error::decode),
            PgValue::Text(s) => i64::from_str(s).map_err(Error::decode),
        }
    }
}
