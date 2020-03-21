use std::convert::TryInto;
use std::str::from_utf8;

use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};

use crate::decode::Decode;
use crate::encode::Encode;
use crate::mysql::protocol::TypeId;
use crate::mysql::types::MySqlTypeInfo;
use crate::mysql::{MySql, MySqlValue};
use crate::types::Type;
use crate::Error;

impl Type<MySql> for u8 {
    fn type_info() -> MySqlTypeInfo {
        MySqlTypeInfo::unsigned(TypeId::TINY_INT)
    }
}

impl Encode<MySql> for u8 {
    fn encode(&self, buf: &mut Vec<u8>) {
        let _ = buf.write_u8(*self);
    }
}

impl<'de> Decode<'de, MySql> for u8 {
    fn decode(value: Option<MySqlValue<'de>>) -> crate::Result<MySql, Self> {
        match value.try_into()? {
            MySqlValue::Binary(mut buf) => buf.read_u8().map_err(Into::into),

            MySqlValue::Text(s) => from_utf8(s)
                .map_err(Error::decode)?
                .parse()
                .map_err(Error::decode),
        }
    }
}

impl Type<MySql> for u16 {
    fn type_info() -> MySqlTypeInfo {
        MySqlTypeInfo::unsigned(TypeId::SMALL_INT)
    }
}

impl Encode<MySql> for u16 {
    fn encode(&self, buf: &mut Vec<u8>) {
        let _ = buf.write_u16::<LittleEndian>(*self);
    }
}

impl<'de> Decode<'de, MySql> for u16 {
    fn decode(value: Option<MySqlValue<'de>>) -> crate::Result<MySql, Self> {
        match value.try_into()? {
            MySqlValue::Binary(mut buf) => buf.read_u16::<LittleEndian>().map_err(Into::into),

            MySqlValue::Text(s) => from_utf8(s)
                .map_err(Error::decode)?
                .parse()
                .map_err(Error::decode),
        }
    }
}

impl Type<MySql> for u32 {
    fn type_info() -> MySqlTypeInfo {
        MySqlTypeInfo::unsigned(TypeId::INT)
    }
}

impl Encode<MySql> for u32 {
    fn encode(&self, buf: &mut Vec<u8>) {
        let _ = buf.write_u32::<LittleEndian>(*self);
    }
}

impl<'de> Decode<'de, MySql> for u32 {
    fn decode(value: Option<MySqlValue<'de>>) -> crate::Result<MySql, Self> {
        match value.try_into()? {
            MySqlValue::Binary(mut buf) => buf.read_u32::<LittleEndian>().map_err(Into::into),

            MySqlValue::Text(s) => from_utf8(s)
                .map_err(Error::decode)?
                .parse()
                .map_err(Error::decode),
        }
    }
}

impl Type<MySql> for u64 {
    fn type_info() -> MySqlTypeInfo {
        MySqlTypeInfo::unsigned(TypeId::BIG_INT)
    }
}

impl Encode<MySql> for u64 {
    fn encode(&self, buf: &mut Vec<u8>) {
        let _ = buf.write_u64::<LittleEndian>(*self);
    }
}

impl<'de> Decode<'de, MySql> for u64 {
    fn decode(value: Option<MySqlValue<'de>>) -> crate::Result<MySql, Self> {
        match value.try_into()? {
            MySqlValue::Binary(mut buf) => buf.read_u64::<LittleEndian>().map_err(Into::into),

            MySqlValue::Text(s) => from_utf8(s)
                .map_err(Error::decode)?
                .parse()
                .map_err(Error::decode),
        }
    }
}
