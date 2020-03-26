use crate::decode::Decode;
use crate::encode::Encode;
use crate::postgres::protocol::TypeId;
use crate::postgres::types::PgTypeInfo;
use crate::postgres::{PgData, PgValue, Postgres};
use crate::types::Type;

impl Type<Postgres> for [u8] {
    fn type_info() -> PgTypeInfo {
        PgTypeInfo::new(TypeId::BYTEA, "BYTEA")
    }
}

impl Type<Postgres> for [&'_ [u8]] {
    fn type_info() -> PgTypeInfo {
        PgTypeInfo::new(TypeId::ARRAY_BYTEA, "BYTEA[]")
    }
}

impl Type<Postgres> for Vec<&'_ [u8]> {
    fn type_info() -> PgTypeInfo {
        <&'_ [u8] as Type<Postgres>>::type_info()
    }
}

impl Type<Postgres> for Vec<u8> {
    fn type_info() -> PgTypeInfo {
        <[u8] as Type<Postgres>>::type_info()
    }
}

impl Encode<Postgres> for [u8] {
    fn encode(&self, buf: &mut Vec<u8>) {
        buf.extend_from_slice(self);
    }
}

impl Encode<Postgres> for Vec<u8> {
    fn encode(&self, buf: &mut Vec<u8>) {
        <[u8] as Encode<Postgres>>::encode(self, buf);
    }
}

impl<'de> Decode<'de, Postgres> for Vec<u8> {
    fn decode(value: PgValue<'de>) -> crate::Result<Self> {
        match value.try_get()? {
            PgData::Binary(buf) => Ok(buf.to_vec()),
            PgData::Text(s) => {
                // BYTEA is formatted as \x followed by hex characters
                hex::decode(&s[2..]).map_err(crate::Error::decode)
            }
        }
    }
}

impl<'de> Decode<'de, Postgres> for &'de [u8] {
    fn decode(value: PgValue<'de>) -> crate::Result<Self> {
        match value.try_get()? {
            PgData::Binary(buf) => Ok(buf),
            PgData::Text(_s) => Err(crate::Error::Decode(
                "unsupported decode to `&[u8]` of BYTEA in a simple query; \
                    use a prepared query or decode to `Vec<u8>`"
                    .into(),
            )),
        }
    }
}
