use byteorder::LittleEndian;

use crate::decode::{Decode, DecodeError};
use crate::encode::Encode;
use crate::io::{Buf, BufMut};
use crate::mysql::protocol::Type;
use crate::mysql::types::MySqlTypeMetadata;
use crate::mysql::MySql;
use crate::types::HasSqlType;

impl HasSqlType<u8> for MySql {
    #[inline]
    fn metadata() -> MySqlTypeMetadata {
        MySqlTypeMetadata::unsigned(Type::TINY)
    }
}

impl Encode<MySql> for u8 {
    fn encode(&self, buf: &mut Vec<u8>) {
        buf.push(*self);
    }
}

impl Decode<MySql> for u8 {
    fn decode(buf: &[u8]) -> Result<Self, DecodeError> {
        Ok(buf[0])
    }
}

impl HasSqlType<u16> for MySql {
    #[inline]
    fn metadata() -> MySqlTypeMetadata {
        MySqlTypeMetadata::unsigned(Type::SHORT)
    }
}

impl Encode<MySql> for u16 {
    fn encode(&self, buf: &mut Vec<u8>) {
        buf.put_u16::<LittleEndian>(*self);
    }
}

impl Decode<MySql> for u16 {
    fn decode(mut buf: &[u8]) -> Result<Self, DecodeError> {
        buf.get_u16::<LittleEndian>().map_err(Into::into)
    }
}

impl HasSqlType<u32> for MySql {
    #[inline]
    fn metadata() -> MySqlTypeMetadata {
        MySqlTypeMetadata::unsigned(Type::LONG)
    }
}

impl Encode<MySql> for u32 {
    fn encode(&self, buf: &mut Vec<u8>) {
        buf.put_u32::<LittleEndian>(*self);
    }
}

impl Decode<MySql> for u32 {
    fn decode(mut buf: &[u8]) -> Result<Self, DecodeError> {
        buf.get_u32::<LittleEndian>().map_err(Into::into)
    }
}

impl HasSqlType<u64> for MySql {
    #[inline]
    fn metadata() -> MySqlTypeMetadata {
        MySqlTypeMetadata::unsigned(Type::LONGLONG)
    }
}

impl Encode<MySql> for u64 {
    fn encode(&self, buf: &mut Vec<u8>) {
        buf.put_u64::<LittleEndian>(*self);
    }
}

impl Decode<MySql> for u64 {
    fn decode(mut buf: &[u8]) -> Result<Self, DecodeError> {
        buf.get_u64::<LittleEndian>().map_err(Into::into)
    }
}
