use super::{Postgres, PostgresTypeFormat, PostgresTypeMetadata};
use crate::{
    deserialize::FromSql,
    serialize::{IsNull, ToSql},
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

impl ToSql<Postgres> for bool {
    #[inline]
    fn to_sql(&self, buf: &mut Vec<u8>) -> IsNull {
        buf.push(*self as u8);

        IsNull::No
    }
}

impl FromSql<Postgres> for bool {
    #[inline]
    fn from_sql(buf: Option<&[u8]>) -> Self {
        // TODO: Handle optionals
        buf.unwrap()[0] != 0
    }
}

// TODO: #[derive(SqlType)]
// pub struct Bool(pub bool);
