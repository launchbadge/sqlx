use crate::decode::Decode;
use crate::encode::Encode;
use crate::postgres::types::{PgJsonb, PgTypeInfo};
use crate::postgres::{PgValue, Postgres};
use crate::types::{Json, Type};
use serde::{Deserialize, Serialize};
use serde_json::value::RawValue as JsonRawValue;
use serde_json::Value as JsonValue;

// <https://www.postgresql.org/docs/12/datatype-json.html>

// In general, most applications should prefer to store JSON data as jsonb,
// unless there are quite specialized needs, such as legacy assumptions
// about ordering of object keys.

impl Type<Postgres> for JsonValue {
    fn type_info() -> PgTypeInfo {
        <PgJsonb<Self> as Type<Postgres>>::type_info()
    }
}

impl Encode<Postgres> for JsonValue {
    fn encode(&self, buf: &mut Vec<u8>) {
        PgJsonb(self).encode(buf)
    }
}

impl<'de> Decode<'de, Postgres> for JsonValue {
    fn decode(value: Option<PgValue<'de>>) -> crate::Result<Postgres, Self> {
        <PgJsonb<Self> as Decode<Postgres>>::decode(value).map(|item| item.0)
    }
}

impl Type<Postgres> for &'_ JsonRawValue {
    fn type_info() -> PgTypeInfo {
        <PgJsonb<Self> as Type<Postgres>>::type_info()
    }
}

impl Encode<Postgres> for &'_ JsonRawValue {
    fn encode(&self, buf: &mut Vec<u8>) {
        PgJsonb(self).encode(buf)
    }
}

impl<'de> Decode<'de, Postgres> for &'de JsonRawValue {
    fn decode(value: Option<PgValue<'de>>) -> crate::Result<Postgres, Self> {
        <PgJsonb<Self> as Decode<Postgres>>::decode(value).map(|item| item.0)
    }
}

impl<T> Type<Postgres> for Json<T> {
    fn type_info() -> PgTypeInfo {
        <PgJsonb<T> as Type<Postgres>>::type_info()
    }
}

impl<T> Encode<Postgres> for Json<T>
where
    T: Serialize,
{
    fn encode(&self, buf: &mut Vec<u8>) {
        PgJsonb(&self.0).encode(buf)
    }
}

impl<'de, T> Decode<'de, Postgres> for Json<T>
where
    T: 'de,
    T: Deserialize<'de>,
{
    fn decode(value: Option<PgValue<'de>>) -> crate::Result<Postgres, Self> {
        <PgJsonb<T> as Decode<Postgres>>::decode(value).map(|item| Self(item.0))
    }
}
