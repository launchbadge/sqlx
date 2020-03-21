use std::convert::TryInto;
use std::str::FromStr;

use uuid::Uuid;

use crate::decode::Decode;
use crate::encode::Encode;
use crate::postgres::protocol::TypeId;
use crate::postgres::row::PgValue;
use crate::postgres::types::PgTypeInfo;
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
    fn decode(value: Option<PgValue<'de>>) -> crate::Result<Postgres, Self> {
        match value.try_into()? {
            PgValue::Binary(buf) => Uuid::from_slice(buf).map_err(|err| crate::Error::decode(err)),
            PgValue::Text(s) => Uuid::from_str(s).map_err(|err| crate::Error::decode(err)),
        }
    }
}
