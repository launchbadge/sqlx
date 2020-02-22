use crate::decode::{Decode, DecodeError};
use crate::encode::Encode;
use crate::io::{Buf, BufMut};
use crate::postgres::protocol::TypeId;
use crate::postgres::types::PgTypeInfo;
use crate::postgres::Postgres;
use crate::types::HasSqlType;
use serde::{Deserialize, Serialize};
use serde_json::Value;

impl HasSqlType<Value> for Postgres {
    fn type_info() -> PgTypeInfo {
        PgTypeInfo::new(TypeId::JSON)
    }
}

impl Encode<Postgres> for Value {
    fn encode(&self, buf: &mut Vec<u8>) {
        Json(self).encode(buf)
    }
}

impl Decode<Postgres> for Value {
    fn decode(buf: &[u8]) -> Result<Self, DecodeError> {
        let Json(item) = Decode::decode(buf)?;
        Ok(item)
    }
}

#[derive(Debug, PartialEq)]
pub struct Json<T>(pub T);

impl<T> HasSqlType<Json<T>> for Postgres {
    fn type_info() -> PgTypeInfo {
        PgTypeInfo::new(TypeId::JSON)
    }
}

impl<T> Encode<Postgres> for Json<T>
where
    T: Serialize,
{
    fn encode(&self, buf: &mut Vec<u8>) {
        serde_json::to_writer(buf, &self.0)
            .expect("failed to serialize json for encoding to database");
    }
}

impl<T> Decode<Postgres> for Json<T>
where
    T: for<'a> Deserialize<'a>,
{
    fn decode(buf: &[u8]) -> Result<Self, DecodeError> {
        let item = serde_json::from_slice(buf)?;
        Ok(Json(item))
    }
}

#[derive(Debug, PartialEq)]
pub struct Jsonb<T>(pub T);

impl<T> HasSqlType<Jsonb<T>> for Postgres {
    fn type_info() -> PgTypeInfo {
        PgTypeInfo::new(TypeId::JSONB)
    }
}

impl<T> Encode<Postgres> for Jsonb<T>
where
    T: Serialize,
{
    fn encode(&self, buf: &mut Vec<u8>) {
        // TODO: I haven't been figure out what this byte is, but it is required or else we get the error:
        // Error: unsupported jsonb version number 34
        buf.put_u8(1);

        serde_json::to_writer(buf, &self.0)
            .expect("failed to serialize json for encoding to database");
    }
}

impl<T> Decode<Postgres> for Jsonb<T>
where
    T: for<'a> Deserialize<'a>,
{
    fn decode(mut buf: &[u8]) -> Result<Self, DecodeError> {
        // TODO: I don't know what this byte is, similarly to Encode
        let _ = buf.get_u8()?;

        let item = serde_json::from_slice(buf)?;
        Ok(Jsonb(item))
    }
}
