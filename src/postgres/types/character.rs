use crate::types::{FromSql, Text, ToSql, ToSqlAs};

impl ToSql for &'_ str {
    type Type = Text;
}

impl ToSqlAs<Text> for &'_ str {
    #[inline]
    fn to_sql(self, buf: &mut Vec<u8>) {
        buf.extend_from_slice(self.as_bytes());
    }
}

impl ToSql for String {
    type Type = Text;
}

impl ToSqlAs<Text> for String {
    #[inline]
    fn to_sql(self, buf: &mut Vec<u8>) {
        buf.extend_from_slice(self.as_bytes());
    }
}

impl FromSql<Text> for String {
    #[inline]
    fn from_sql(buf: &[u8]) -> Self {
        // Using lossy here as it should be impossible to get non UTF8 data here
        String::from_utf8_lossy(buf).into()
    }
}
