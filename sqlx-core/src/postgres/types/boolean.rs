use super::{Postgres, PostgresTypeFormat, PostgresTypeMetadata};
use crate::{
    decode::Decode,
    encode::{IsNull, Encode},
    types::HasSqlType,
};

impl HasSqlType<bool> for Postgres {
    fn metadata() -> PostgresTypeMetadata {
        PostgresTypeMetadata {
            format: PostgresTypeFormat::Binary,
            oid: 16,
            array_oid: 1000,
        }
    }
}

impl Encode<Postgres> for bool {
    #[inline]
    fn encode(&self, buf: &mut Vec<u8>) -> IsNull {
        buf.push(*self as u8);

        IsNull::No
    }
}

impl Decode<Postgres> for bool {
    #[inline]
    fn decode(buf: Option<&[u8]>) -> Self {
        // TODO: Handle optionals
        buf.unwrap()[0] != 0
    }
}

// TODO: #[derive(SqlType)]
// pub struct Bool(pub bool);
