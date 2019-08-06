use super::TypeMetadata;
use crate::{
    deserialize::FromSql,
    postgres::Postgres,
    serialize::{IsNull, ToSql},
    types::{AsSql, SqlType},
};

pub struct Bool;

impl SqlType<Postgres> for Bool {
    fn metadata() -> TypeMetadata {
        TypeMetadata {
            oid: 16,
            array_oid: 1000,
        }
    }
}

impl AsSql<Postgres> for bool {
    type Type = Bool;
}

impl ToSql<Postgres, Bool> for bool {
    #[inline]
    fn to_sql(self, buf: &mut Vec<u8>) -> IsNull {
        buf.push(self as u8);

        IsNull::No
    }
}

impl FromSql<Postgres, Bool> for bool {
    #[inline]
    fn from_sql(buf: Option<&[u8]>) -> Self {
        // TODO: Handle optionals
        buf.unwrap()[0] != 0
    }
}
