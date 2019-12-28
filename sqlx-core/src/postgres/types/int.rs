use crate::decode::{Decode, DecodeError};
use crate::encode::Encode;
use crate::postgres::types::PgTypeMetadata;
use crate::postgres::Postgres;
use crate::types::HasSqlType;
use byteorder::{ByteOrder, NetworkEndian};

impl HasSqlType<i16> for Postgres {
    fn metadata() -> PgTypeMetadata {
        PgTypeMetadata::binary(21, 1005)
    }
}

impl Encode<Postgres> for i16 {
    fn encode(&self, buf: &mut Vec<u8>) {
        buf.extend_from_slice(&self.to_be_bytes());
    }
}

impl Decode<Postgres> for i16 {
    fn decode(buf: &[u8]) -> Result<Self, DecodeError> {
        Ok(NetworkEndian::read_i16(buf))
    }
}

impl HasSqlType<i32> for Postgres {
    fn metadata() -> PgTypeMetadata {
        PgTypeMetadata::binary(23, 1007)
    }
}

impl Encode<Postgres> for i32 {
    fn encode(&self, buf: &mut Vec<u8>) {
        buf.extend_from_slice(&self.to_be_bytes());
    }
}

impl Decode<Postgres> for i32 {
    fn decode(buf: &[u8]) -> Result<Self, DecodeError> {
        Ok(NetworkEndian::read_i32(buf))
    }
}

impl HasSqlType<i64> for Postgres {
    fn metadata() -> PgTypeMetadata {
        PgTypeMetadata::binary(20, 1016)
    }
}

impl Encode<Postgres> for i64 {
    fn encode(&self, buf: &mut Vec<u8>) {
        buf.extend_from_slice(&self.to_be_bytes());
    }
}

impl Decode<Postgres> for i64 {
    fn decode(buf: &[u8]) -> Result<Self, DecodeError> {
        Ok(NetworkEndian::read_i64(buf))
    }
}
