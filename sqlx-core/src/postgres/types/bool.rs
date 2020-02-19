use crate::decode::{Decode, DecodeError};
use crate::encode::Encode;
use crate::postgres::protocol::TypeId;
use crate::postgres::types::PgTypeInfo;
use crate::postgres::Postgres;
use crate::types::Type;

impl Type<Postgres> for bool {
    fn type_info() -> PgTypeInfo {
        PgTypeInfo::new(TypeId::BOOL)
    }
}

impl Type<Postgres> for [bool] {
    fn type_info() -> PgTypeInfo {
        PgTypeInfo::new(TypeId::ARRAY_BOOL)
    }
}

impl Encode<Postgres> for bool {
    fn encode(&self, buf: &mut Vec<u8>) {
        buf.push(*self as u8);
    }
}

impl Decode<Postgres> for bool {
    fn decode(buf: &[u8]) -> Result<Self, DecodeError> {
        buf.get(0).map(|&b| b != 0).ok_or_else(|| {
            DecodeError::Message(Box::new("Expected minimum 1 byte but received none."))
        })
    }
}
