use super::TypeMetadata;
use crate::{
    deserialize::FromSql,
    postgres::Postgres,
    serialize::{IsNull, ToSql},
    types::{AsSql, SqlType, Text},
};

impl SqlType<Postgres> for Text {
    fn metadata() -> TypeMetadata {
        TypeMetadata {
            oid: 25,
            array_oid: 1009,
        }
    }
}

impl AsSql<Postgres> for &'_ str {
    type Type = Text;
}

impl ToSql<Postgres, Text> for &'_ str {
    #[inline]
    fn to_sql(self, buf: &mut Vec<u8>) -> IsNull {
        buf.extend_from_slice(self.as_bytes());

        IsNull::No
    }
}

impl AsSql<Postgres> for String {
    type Type = Text;
}

impl ToSql<Postgres, Text> for String {
    #[inline]
    fn to_sql(self, buf: &mut Vec<u8>) -> IsNull {
        buf.extend_from_slice(self.as_bytes());

        IsNull::No
    }
}

impl FromSql<Postgres, Text> for String {
    #[inline]
    fn from_sql(buf: Option<&[u8]>) -> Self {
        // TODO: Handle optionals
        // Using lossy here as it should be impossible to get non UTF8 data here
        String::from_utf8_lossy(buf.unwrap()).into()
    }
}
