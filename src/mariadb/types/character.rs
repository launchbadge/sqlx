use super::{MariaDb, MariaDbTypeMetadata};
use crate::{
    deserialize::FromSql,
    mariadb::protocol::{FieldType, ParameterFlag},
    serialize::{IsNull, ToSql},
    types::HasSqlType,
};
use std::str;

impl HasSqlType<&'_ str> for MariaDb {
    #[inline]
    fn metadata() -> MariaDbTypeMetadata {
        MariaDbTypeMetadata {
            // MYSQL_TYPE_VAR_STRING
            field_type: FieldType(253),
            param_flag: ParameterFlag::empty(),
        }
    }
}

impl HasSqlType<String> for MariaDb {
    #[inline]
    fn metadata() -> MariaDbTypeMetadata {
        <MariaDb as HasSqlType<&str>>::metadata()
    }
}

impl ToSql<MariaDb> for &'_ str {
    #[inline]
    fn to_sql(self, buf: &mut Vec<u8>) -> IsNull {
        buf.extend_from_slice(self.as_bytes());

        IsNull::No
    }
}

impl ToSql<MariaDb> for String {
    #[inline]
    fn to_sql(self, buf: &mut Vec<u8>) -> IsNull {
        self.as_str().to_sql(buf)
    }
}

impl FromSql<MariaDb> for String {
    #[inline]
    fn from_sql(buf: Option<&[u8]>) -> Self {
        // TODO: Handle nulls

        let s = if cfg!(debug_assertions) {
            str::from_utf8(buf.unwrap()).expect("mariadb returned non UTF-8 data for VAR_STRING")
        } else {
            // TODO: Determine how to treat string if different collation
            unsafe { str::from_utf8_unchecked(buf.unwrap()) }
        };

        s.to_owned()
    }
}
