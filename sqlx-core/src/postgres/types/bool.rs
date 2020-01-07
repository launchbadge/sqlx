use crate::decode::{Decode, DecodeError};
use crate::encode::Encode;
use crate::postgres::types::PgTypeMetadata;
use crate::postgres::Postgres;
use crate::types::HasSqlType;

impl HasSqlType<bool> for Postgres {
    fn metadata() -> PgTypeMetadata {
        PgTypeMetadata::binary(16, 100)
    }
}

impl Encode<Postgres> for bool {
    fn encode(&self, buf: &mut Vec<u8>) {
        buf.push(*self as u8);
    }
}

impl Decode<Postgres> for bool {
    fn decode(buf: &[u8]) -> Result<Self, DecodeError> {
        match buf.len() {
            0 => Err(DecodeError::Message(Box::new(
                "Expected minimum 1 byte but received none.",
            ))),
            _ => Ok(buf[0] != 0),
        }
    }
}
