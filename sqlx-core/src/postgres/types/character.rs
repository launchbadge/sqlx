use super::{Postgres, PostgresTypeFormat, PostgresTypeMetadata};
use crate::{
    deserialize::FromSql,
    serialize::{IsNull, ToSql},
    types::HasSqlType,
};
use std::str;

impl HasSqlType<str> for Postgres {
    #[inline]
    fn metadata() -> PostgresTypeMetadata {
        PostgresTypeMetadata {
            format: PostgresTypeFormat::Binary,
            oid: 25,
            array_oid: 1009,
        }
    }
}

impl HasSqlType<String> for Postgres {
    #[inline]
    fn metadata() -> PostgresTypeMetadata {
        <Postgres as HasSqlType<str>>::metadata()
    }
}

impl ToSql<Postgres> for str {
    #[inline]
    fn to_sql(&self, buf: &mut Vec<u8>) -> IsNull {
        buf.extend_from_slice(self.as_bytes());

        IsNull::No
    }
}

impl ToSql<Postgres> for String {
    #[inline]
    fn to_sql(&self, buf: &mut Vec<u8>) -> IsNull {
        <str as ToSql<Postgres>>::to_sql(self.as_str(), buf)
    }
}

impl FromSql<Postgres> for String {
    #[inline]
    fn from_sql(buf: Option<&[u8]>) -> Self {
        // TODO: Handle nulls

        let s = if cfg!(debug_assertions) {
            str::from_utf8(buf.unwrap()).expect("postgres returned non UTF-8 data for TEXT")
        } else {
            // SAFE: Postgres is required to return UTF-8 data
            unsafe { str::from_utf8_unchecked(buf.unwrap()) }
        };

        s.to_owned()
    }
}
