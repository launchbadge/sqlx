use std::str::FromStr;

use uuid::Uuid;

use crate::decode::Decode;
use crate::encode::Encode;
use crate::postgres::protocol::TypeId;
use crate::postgres::types::PgTypeInfo;
use crate::postgres::value::{PgData, PgValue};
use crate::postgres::Postgres;
use crate::types::Type;

impl Type<Postgres> for Uuid {
    fn type_info() -> PgTypeInfo {
        PgTypeInfo::new(TypeId::UUID, "UUID")
    }
}

impl Type<Postgres> for [Uuid] {
    fn type_info() -> PgTypeInfo {
        PgTypeInfo::new(TypeId::ARRAY_UUID, "UUID[]")
    }
}

impl Type<Postgres> for Vec<Uuid> {
    fn type_info() -> PgTypeInfo {
        <[Uuid] as Type<Postgres>>::type_info()
    }
}

impl Encode<Postgres> for Uuid {
    fn encode(&self, buf: &mut Vec<u8>) {
        buf.extend_from_slice(self.as_bytes());
    }
}

impl<'de> Decode<'de, Postgres> for Uuid {
    fn decode(value: PgValue<'de>) -> crate::Result<Self> {
        match value.try_get()? {
            PgData::Binary(buf) => Uuid::from_slice(buf).map_err(crate::Error::decode),
            PgData::Text(s) => Uuid::from_str(s).map_err(crate::Error::decode),
        }
    }
}
