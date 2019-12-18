use super::{MySql, MariaDbTypeMetadata};
use crate::{
    decode::Decode,
    encode::{Encode, IsNull},
    mysql::protocol::{FieldType, ParameterFlag},
    types::HasSqlType,
};
use std::str;
use crate::mysql::io::BufMutExt;
use byteorder::LittleEndian;

impl HasSqlType<str> for MySql {
    #[inline]
    fn metadata() -> MariaDbTypeMetadata {
        MariaDbTypeMetadata {
            // MYSQL_TYPE_VAR_STRING
            field_type: FieldType::MYSQL_TYPE_VAR_STRING,
            param_flag: ParameterFlag::empty(),
        }
    }
}

impl Encode<MySql> for str {
    #[inline]
    fn encode(&self, buf: &mut Vec<u8>) -> IsNull {
        buf.put_str_lenenc::<LittleEndian>(self);

        IsNull::No
    }
}

impl HasSqlType<String> for MySql {
    #[inline]
    fn metadata() -> MariaDbTypeMetadata {
        <MySql as HasSqlType<&str>>::metadata()
    }
}

impl Encode<MySql> for String {
    #[inline]
    fn encode(&self, buf: &mut Vec<u8>) -> IsNull {
        <str as Encode<MySql>>::encode(self.as_str(), buf)
    }
}

impl Decode<MySql> for String {
    #[inline]
    fn decode(buf: Option<&[u8]>) -> Self {
        // TODO: Handle nulls

        let s = if cfg!(debug_assertions) {
            str::from_utf8(buf.unwrap()).expect("mysql returned non UTF-8 data for VAR_STRING")
        } else {
            // TODO: Determine how to treat string if different collation
            unsafe { str::from_utf8_unchecked(buf.unwrap()) }
        };

        s.to_owned()
    }
}
