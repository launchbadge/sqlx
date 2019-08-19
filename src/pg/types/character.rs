use super::{Pg, PgTypeMetadata};
use crate::{
    deserialize::FromSql,
    serialize::{IsNull, ToSql},
    types::{AsSqlType, HasSqlType, Text},
};
use std::str;

impl HasSqlType<Text> for Pg {
    fn metadata() -> PgTypeMetadata {
        PgTypeMetadata {
            oid: 25,
            array_oid: 1009,
        }
    }
}

impl AsSqlType<Pg> for &'_ str {
    type SqlType = Text;
}

impl ToSql<Text, Pg> for &'_ str {
    #[inline]
    fn to_sql(self, buf: &mut Vec<u8>) -> IsNull {
        buf.extend_from_slice(self.as_bytes());

        IsNull::No
    }
}

impl AsSqlType<Pg> for String {
    type SqlType = Text;
}

impl ToSql<Text, Pg> for String {
    #[inline]
    fn to_sql(self, buf: &mut Vec<u8>) -> IsNull {
        buf.extend_from_slice(self.as_bytes());

        IsNull::No
    }
}

impl FromSql<Text, Pg> for String {
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
