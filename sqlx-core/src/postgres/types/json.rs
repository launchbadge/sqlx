use crate::decode::Decode;
use crate::encode::Encode;
use crate::io::{Buf, BufMut};
use crate::postgres::protocol::TypeId;
use crate::postgres::{PgData, PgTypeInfo, PgValue, Postgres};
use crate::types::{Json, Type};
use crate::value::RawValue;
use serde::{Deserialize, Serialize};
use serde_json::value::RawValue as JsonRawValue;
use serde_json::Value as JsonValue;

// <https://www.postgresql.org/docs/12/datatype-json.html>

// In general, most applications should prefer to store JSON data as jsonb,
// unless there are quite specialized needs, such as legacy assumptions
// about ordering of object keys.

impl Type<Postgres> for JsonValue {
    fn type_info() -> PgTypeInfo {
        <Json<Self> as Type<Postgres>>::type_info()
    }
}

impl Encode<Postgres> for JsonValue {
    fn encode(&self, buf: &mut Vec<u8>) {
        Json(self).encode(buf)
    }
}

impl<'de> Decode<'de, Postgres> for JsonValue {
    fn decode(value: PgValue<'de>) -> crate::Result<Postgres, Self> {
        <Json<Self> as Decode<Postgres>>::decode(value).map(|item| item.0)
    }
}

impl Type<Postgres> for &'_ JsonRawValue {
    fn type_info() -> PgTypeInfo {
        <Json<Self> as Type<Postgres>>::type_info()
    }
}

impl Encode<Postgres> for &'_ JsonRawValue {
    fn encode(&self, buf: &mut Vec<u8>) {
        Json(self).encode(buf)
    }
}

impl<'de> Decode<'de, Postgres> for &'de JsonRawValue {
    fn decode(value: PgValue<'de>) -> crate::Result<Postgres, Self> {
        <Json<Self> as Decode<Postgres>>::decode(value).map(|item| item.0)
    }
}

impl<T> Type<Postgres> for Json<T> {
    fn type_info() -> PgTypeInfo {
        PgTypeInfo::new(TypeId::JSONB, "JSONB")
    }
}

impl<T> Encode<Postgres> for Json<T>
where
    T: Serialize,
{
    fn encode(&self, buf: &mut Vec<u8>) {
        // JSONB version (as of 2020-03-20  )
        buf.put_u8(1);

        serde_json::to_writer(buf, &self.0)
            .expect("failed to serialize json for encoding to database");
    }
}

impl<'de, T> Decode<'de, Postgres> for Json<T>
where
    T: 'de,
    T: Deserialize<'de>,
{
    fn decode(value: PgValue<'de>) -> crate::Result<Postgres, Self> {
        (match value.try_get()? {
            PgData::Text(s) => serde_json::from_str(s),
            PgData::Binary(mut buf) => {
                if value.type_info().as_ref().map(|info| info.id) == Some(TypeId::JSONB) {
                    let version = buf.get_u8()?;

                    assert_eq!(
                        version, 1,
                        "unsupported JSONB format version {}; please open an issue",
                        version
                    );
                }

                serde_json::from_slice(buf)
            }
        })
        .map(Json)
        .map_err(crate::Error::decode)
    }
}
