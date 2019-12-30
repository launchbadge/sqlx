use crate::decode::{Decode, DecodeError};
use crate::encode::Encode;
use crate::postgres::types::PgTypeMetadata;
use crate::postgres::Postgres;
use crate::types::HasSqlType;

impl HasSqlType<[u8]> for Postgres {
    fn metadata() -> PgTypeMetadata {
        PgTypeMetadata::binary(17, 1001)
    }
}

impl HasSqlType<Vec<u8>> for Postgres {
    fn metadata() -> Self::TypeMetadata {
        <Postgres as HasSqlType<[u8]>>::metadata()
    }
}

impl Encode<Postgres> for [u8] {
    fn encode(&self, buf: &mut Vec<u8>) {
        buf.extend_from_slice(self);
    }

    fn size_hint(&self) -> usize {
        self.len()
    }
}

impl Encode<Postgres> for Vec<u8> {
    fn encode(&self, buf: &mut Vec<u8>) {
        <[u8] as Encode<Postgres>>::encode(self, buf);
    }

    fn size_hint(&self) -> usize {
        self.len()
    }
}

impl Decode<Postgres> for Vec<u8> {
    fn decode(buf: &[u8]) -> Result<Self, DecodeError> {
        Ok(buf.to_vec())
    }
}
