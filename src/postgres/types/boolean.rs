use crate::types::{SqlType, ToSql, ToSqlAs, FromSql};

pub struct Bool;

impl SqlType for Bool {
    const OID: u32 = 16;
}

impl ToSql for bool {
    type Type = Bool;
}

impl ToSqlAs<Bool> for bool {
    #[inline]
    fn to_sql(self, buf: &mut Vec<u8>) {
        buf.push(self as u8);
    }
}

impl FromSql<Bool> for bool {
    #[inline]
    fn from_sql(buf: &[u8]) -> Self {
        buf[0] != 0
    }
}
