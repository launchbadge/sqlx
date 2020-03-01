use uuid::Uuid;

use crate::decode::Decode;
use crate::encode::Encode;
use crate::postgres::protocol::TypeId;
use crate::postgres::types::PgTypeInfo;
use crate::postgres::Postgres;
use crate::types::Type;

impl Type<Postgres> for Uuid {
    fn type_info() -> PgTypeInfo {
        PgTypeInfo::new(TypeId::UUID)
    }
}

impl Type<Postgres> for [Uuid] {
    fn type_info() -> PgTypeInfo {
        PgTypeInfo::new(TypeId::ARRAY_UUID)
    }
}

impl Encode<Postgres> for Uuid {
    fn encode(&self, buf: &mut Vec<u8>) {
        buf.extend_from_slice(self.as_bytes());
    }
}

impl<'de> Decode<'de, Postgres> for Uuid {
    fn decode(buf: &'de [u8]) -> crate::Result<Self> {
        Uuid::from_slice(buf).map_err(|err| crate::Error::Decode(Box::new(err)))
    }
}
