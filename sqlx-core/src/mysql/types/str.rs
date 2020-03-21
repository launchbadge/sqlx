use std::str;

use byteorder::LittleEndian;

use crate::decode::Decode;
use crate::encode::Encode;
use crate::mysql::io::BufMutExt;
use crate::mysql::protocol::TypeId;
use crate::mysql::types::MySqlTypeInfo;
use crate::mysql::{MySql, MySqlValue};
use crate::types::Type;
use std::convert::TryInto;
use std::str::from_utf8;

impl Type<MySql> for str {
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

impl Type<MySql> for String {
    fn type_info() -> MySqlTypeInfo {
        <str as Type<MySql>>::type_info()
    }
}

impl Encode<MySql> for String {
    fn encode(&self, buf: &mut Vec<u8>) {
        <str as Encode<MySql>>::encode(self.as_str(), buf)
    }
}

impl<'de> Decode<'de, MySql> for &'de str {
    fn decode(value: Option<MySqlValue<'de>>) -> crate::Result<MySql, Self> {
        match value.try_into()? {
            MySqlValue::Binary(buf) | MySqlValue::Text(buf) => {
                from_utf8(buf).map_err(crate::Error::decode)
            }
        }
    }
}

impl<'de> Decode<'de, MySql> for String {
    fn decode(buf: Option<MySqlValue<'de>>) -> crate::Result<MySql, Self> {
        <&'de str as Decode<MySql>>::decode(buf).map(ToOwned::to_owned)
    }
}
