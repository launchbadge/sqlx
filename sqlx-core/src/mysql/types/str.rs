use std::str;

use byteorder::LittleEndian;

use crate::decode::{Decode, DecodeError};
use crate::encode::Encode;
use crate::mysql::io::{BufExt, BufMutExt};
use crate::mysql::protocol::TypeId;
use crate::mysql::types::MySqlTypeInfo;
use crate::mysql::MySql;
use crate::types::Type;

impl Type<str> for MySql {
    fn type_info() -> MySqlTypeInfo {
        MySqlTypeInfo {
            id: TypeId::TEXT,
            is_binary: false,
            is_unsigned: false,
            char_set: 224, // utf8mb4_unicode_ci
        }
    }
}

impl Encode<MySql> for str {
    fn encode(&self, buf: &mut Vec<u8>) {
        buf.put_str_lenenc::<LittleEndian>(self);
    }
}

// TODO: Do we need the [HasSqlType] for String
impl Type<String> for MySql {
    fn type_info() -> MySqlTypeInfo {
        <Self as Type<&str>>::type_info()
    }
}

impl Encode<MySql> for String {
    fn encode(&self, buf: &mut Vec<u8>) {
        <str as Encode<MySql>>::encode(self.as_str(), buf)
    }
}

impl Decode<MySql> for String {
    fn decode(mut buf: &[u8]) -> Result<Self, DecodeError> {
        Ok(buf
            .get_str_lenenc::<LittleEndian>()?
            .unwrap_or_default()
            .to_owned())
    }
}
