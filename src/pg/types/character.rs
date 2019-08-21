use super::{Pg, PgTypeMetadata};
use crate::{
    deserialize::FromSql,
    serialize::{IsNull, ToSql},
    types::HasSqlType,
};
use std::str;

impl HasSqlType<&'_ str> for Pg {
    #[inline]
    fn metadata() -> PgTypeMetadata {
        PgTypeMetadata {
            oid: 25,
            array_oid: 1009,
        }
    }
}

impl HasSqlType<String> for Pg {
    #[inline]
    fn metadata() -> PgTypeMetadata {
        <Pg as HasSqlType<&str>>::metadata()
    }
}

impl ToSql<Pg> for &'_ str {
    #[inline]
    fn to_sql(self, buf: &mut Vec<u8>) -> IsNull {
        buf.extend_from_slice(self.as_bytes());

        IsNull::No
    }
}

impl ToSql<Pg> for String {
    #[inline]
    fn to_sql(self, buf: &mut Vec<u8>) -> IsNull {
        self.as_str().to_sql(buf)
    }
}

impl FromSql<Pg> for String {
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
