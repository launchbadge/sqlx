use crate::{
    postgres::types::{PostgresTypeFormat, PostgresTypeMetadata},
    encode::IsNull,
    Decode, HasSqlType, Postgres, Encode,
};

impl HasSqlType<[u8]> for Postgres {
    fn metadata() -> Self::TypeMetadata {
        PostgresTypeMetadata {
            format: PostgresTypeFormat::Binary,
            oid: 17,
            array_oid: 1001,
        }
    }
}

impl HasSqlType<Vec<u8>> for Postgres {
    fn metadata() -> Self::TypeMetadata {
        <Postgres as HasSqlType<[u8]>>::metadata()
    }
}

impl Encode<Postgres> for [u8] {
    fn encode(&self, buf: &mut Vec<u8>) -> IsNull {
        buf.extend_from_slice(self);
        IsNull::No
    }
}

impl Encode<Postgres> for Vec<u8> {
    fn encode(&self, buf: &mut Vec<u8>) -> IsNull {
        <[u8] as Encode<Postgres>>::to_sql(self, buf)
    }
}

impl Decode<Postgres> for Vec<u8> {
    fn decode(raw: Option<&[u8]>) -> Self {
        raw.unwrap().into()
    }
}
