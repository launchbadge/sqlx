use uuid::Uuid;

use super::{Postgres, PostgresTypeFormat, PostgresTypeMetadata};
use crate::{
    decode::Decode,
    encode::{IsNull, Encode},
    types::HasSqlType,
};

impl HasSqlType<Uuid> for Postgres {
    fn metadata() -> PostgresTypeMetadata {
        PostgresTypeMetadata {
            format: PostgresTypeFormat::Binary,
            oid: 2950,
            array_oid: 2951,
        }
    }
}

impl Encode<Postgres> for Uuid {
    #[inline]
    fn encode(&self, buf: &mut Vec<u8>) -> IsNull {
        buf.extend_from_slice(self.as_bytes());

        IsNull::No
    }
}

impl Decode<Postgres> for Uuid {
    #[inline]
    fn decode(buf: Option<&[u8]>) -> Self {
        // TODO: Handle optionals, error
        Uuid::from_slice(buf.unwrap()).unwrap()
    }
}
