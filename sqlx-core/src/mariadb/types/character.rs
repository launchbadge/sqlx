use super::{MariaDb, MariaDbTypeMetadata};
use crate::{
    decode::Decode,
    encode::{Encode, IsNull},
    mariadb::protocol::{FieldType, ParameterFlag},
    types::HasSqlType,
};
use std::str;

impl HasSqlType<str> for MariaDb {
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

impl Encode<MariaDb> for str {
    #[inline]
    fn encode(&self, buf: &mut Vec<u8>) -> IsNull {
        buf.extend_from_slice(self.as_bytes());

        IsNull::No
    }
}

impl Encode<MariaDb> for String {
    #[inline]
    fn encode(&self, buf: &mut Vec<u8>) -> IsNull {
        <str as Encode<MariaDb>>::encode(self.as_str(), buf)
    }
}

impl Decode<MariaDb> for String {
    #[inline]
    fn decode(buf: Option<&[u8]>) -> Self {
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
