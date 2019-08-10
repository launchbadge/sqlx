use super::{Pg, PgTypeMetadata};
use crate::{
    deserialize::FromSql,
    serialize::{IsNull, ToSql},
    types::{AsSqlType, HasSqlType},
};

pub struct Bool;

impl HasSqlType<Bool> for Pg {
    fn metadata() -> PgTypeMetadata {
        PgTypeMetadata {
            oid: 16,
            array_oid: 1000,
        }
    }
}

impl AsSqlType<Pg> for bool {
    type SqlType = Bool;
}

impl ToSql<Bool, Pg> for bool {
    #[inline]
    fn to_sql(self, buf: &mut Vec<u8>) -> IsNull {
        buf.push(self as u8);

        IsNull::No
    }
}

impl FromSql<Bool, Pg> for bool {
    #[inline]
    fn from_sql(buf: Option<&[u8]>) -> Self {
        // TODO: Handle optionals
        buf.unwrap()[0] != 0
    }
}
