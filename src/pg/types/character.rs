use super::{Pg, PgTypeMetadata};
use crate::{
    deserialize::FromSql,
    serialize::{IsNull, ToSql},
    types::{AsSqlType, HasSqlType, Text},
};

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
        // TODO: Handle optionals
        // Using lossy here as it should be impossible to get non UTF8 data here
        String::from_utf8_lossy(buf.unwrap()).into()
    }
}
