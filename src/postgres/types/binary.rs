use crate::{
    postgres::types::{PostgresTypeFormat, PostgresTypeMetadata},
    serialize::IsNull,
    FromSql, HasSqlType, Postgres, ToSql,
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

impl ToSql<Postgres> for [u8] {
    fn to_sql(&self, buf: &mut Vec<u8>) -> IsNull {
        buf.extend_from_slice(self);
        IsNull::No
    }
}

impl ToSql<Postgres> for Vec<u8> {
    fn to_sql(&self, buf: &mut Vec<u8>) -> IsNull {
        <[u8] as ToSql<Postgres>>::to_sql(self, buf)
    }
}

impl FromSql<Postgres> for Vec<u8> {
    fn from_sql(raw: Option<&[u8]>) -> Self {
        raw.unwrap().into()
    }
}
