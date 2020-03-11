use byteorder::LittleEndian;

use crate::decode::Decode;
use crate::encode::Encode;
use crate::error::UnexpectedNullError;
use crate::mysql::io::{BufExt, BufMutExt};
use crate::mysql::protocol::TypeId;
use crate::mysql::types::MySqlTypeInfo;
use crate::mysql::{MySql, MySqlValue};
use crate::types::Type;
use std::convert::TryInto;

impl Type<MySql> for [u8] {
    fn type_info() -> MySqlTypeInfo {
        MySqlTypeInfo {
            id: TypeId::TEXT,
            is_binary: true,
            is_unsigned: false,
            char_set: 63, // binary
        }
    }
}

impl Type<MySql> for Vec<u8> {
    fn type_info() -> MySqlTypeInfo {
        <[u8] as Type<MySql>>::type_info()
    }
}

impl Encode<MySql> for [u8] {
    fn encode(&self, buf: &mut Vec<u8>) {
        buf.put_bytes_lenenc::<LittleEndian>(self);
    }
}

impl Encode<MySql> for Vec<u8> {
    fn encode(&self, buf: &mut Vec<u8>) {
        <[u8] as Encode<MySql>>::encode(self, buf);
    }
}

impl<'de> Decode<'de, MySql> for Vec<u8> {
    fn decode(value: Option<MySqlValue<'de>>) -> crate::Result<Self> {
        match value.try_into()? {
            MySqlValue::Binary(mut buf) => {
                let len = buf
                    .get_uint_lenenc::<LittleEndian>()
                    .map_err(crate::Error::decode)?
                    .unwrap_or_default();

                Ok((&buf[..(len as usize)]).to_vec())
            }

            MySqlValue::Text(s) => Ok(s.to_vec()),
        }
    }
}

impl<'de> Decode<'de, MySql> for &'de [u8] {
    fn decode(value: Option<MySqlValue<'de>>) -> crate::Result<Self> {
        match value.try_into()? {
            MySqlValue::Binary(mut buf) => {
                let len = buf
                    .get_uint_lenenc::<LittleEndian>()
                    .map_err(crate::Error::decode)?
                    .unwrap_or_default();

                Ok(&buf[..(len as usize)])
            }

            MySqlValue::Text(s) => Ok(s),
        }
    }
}
