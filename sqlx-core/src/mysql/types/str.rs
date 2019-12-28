use std::str;

use byteorder::LittleEndian;

use crate::decode::{Decode, DecodeError};
use crate::encode::Encode;
use crate::mysql::io::{BufMutExt};
use crate::mysql::protocol::Type;
use crate::mysql::types::MySqlTypeMetadata;
use crate::mysql::MySql;
use crate::types::HasSqlType;

impl HasSqlType<str> for MySql {
    fn metadata() -> MySqlTypeMetadata {
        MySqlTypeMetadata::new(Type::VAR_STRING)
    }
}

impl Encode<MySql> for str {
    fn encode(&self, buf: &mut Vec<u8>) {
        buf.put_str_lenenc::<LittleEndian>(self);
    }
}

impl HasSqlType<String> for MySql {
    fn metadata() -> MySqlTypeMetadata {
        <MySql as HasSqlType<&str>>::metadata()
    }
}

impl Encode<MySql> for String {
    fn encode(&self, buf: &mut Vec<u8>) {
        <str as Encode<MySql>>::encode(self.as_str(), buf)
    }
}

impl Decode<MySql> for String {
    fn decode(mut buf: &[u8]) -> Result<Self, DecodeError> {
        Ok(str::from_utf8(buf)?.to_owned())
    }
}
