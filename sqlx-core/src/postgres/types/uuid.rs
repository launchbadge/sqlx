use crate::decode::{Decode, DecodeError};
use crate::encode::Encode;
use crate::postgres::types::PgTypeMetadata;
use crate::postgres::Postgres;
use crate::types::HasSqlType;
use uuid::Uuid;

impl HasSqlType<Uuid> for Postgres {
    fn metadata() -> PgTypeMetadata {
        PgTypeMetadata::binary(2950, 2951)
    }
}

impl Encode<Postgres> for Uuid {
    fn encode(&self, buf: &mut Vec<u8>) {
        buf.extend_from_slice(self.as_bytes());
    }
}

impl Decode<Postgres> for Uuid {
    fn decode(buf: &[u8]) -> Result<Self, DecodeError> {
        Uuid::from_slice(buf).map_err(|err| DecodeError::Message(Box::new(err)))
    }
}
