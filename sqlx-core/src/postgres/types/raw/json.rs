use crate::decode::Decode;
use crate::encode::Encode;
use crate::io::{Buf, BufMut};
use crate::postgres::protocol::TypeId;
use crate::postgres::types::PgTypeInfo;
use crate::postgres::{PgValue, Postgres};
use crate::types::Type;
use serde::{Deserialize, Serialize};
use std::convert::TryInto;

#[derive(Debug, PartialEq)]
pub struct PgJson<T>(pub T);

impl<T> Type<Postgres> for PgJson<T> {
    fn type_info() -> PgTypeInfo {
        PgTypeInfo::new(TypeId::JSON, "JSON")
    }
}

impl<T> Encode<Postgres> for PgJson<T>
where
    T: Serialize,
{
    fn encode(&self, buf: &mut Vec<u8>) {
        serde_json::to_writer(buf, &self.0)
            .expect("failed to serialize json for encoding to database");
    }
}

impl<'de, T> Decode<'de, Postgres> for PgJson<T>
where
    T: 'de,
    T: Deserialize<'de>,
{
    fn decode(value: Option<PgValue<'de>>) -> crate::Result<Postgres, Self> {
        (match value.try_into()? {
            PgValue::Text(s) => serde_json::from_str(s),
            PgValue::Binary(buf) => serde_json::from_slice(buf),
        })
        .map(PgJson)
        .map_err(crate::Error::decode)
    }
}

// This type has the Pg prefix as it is a postgres-only extension
// unlike the normal Json<T> wrapper
#[derive(Debug, PartialEq)]
pub struct PgJsonb<T>(pub T);

impl<T> Type<Postgres> for PgJsonb<T> {
    fn type_info() -> PgTypeInfo {
        PgTypeInfo::new(TypeId::JSONB, "JSONB")
    }
}

impl<T> Encode<Postgres> for PgJsonb<T>
where
    T: Serialize,
{
    fn encode(&self, buf: &mut Vec<u8>) {
        // JSONB version (as of 2020-03-20)
        buf.put_u8(1);

        serde_json::to_writer(buf, &self.0)
            .expect("failed to serialize json for encoding to database");
    }
}

impl<'de, T> Decode<'de, Postgres> for PgJsonb<T>
where
    T: 'de,
    T: Deserialize<'de>,
{
    fn decode(value: Option<PgValue<'de>>) -> crate::Result<Postgres, Self> {
        (match value.try_into()? {
            PgValue::Text(s) => serde_json::from_str(s),
            PgValue::Binary(mut buf) => {
                let version = buf.get_u8()?;

                assert_eq!(
                    version, 1,
                    "unsupported JSONB format version {}; please open an issue",
                    version
                );

                serde_json::from_slice(buf)
            }
        })
        .map(PgJsonb)
        .map_err(crate::Error::decode)
    }
}
