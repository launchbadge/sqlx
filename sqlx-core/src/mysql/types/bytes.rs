use byteorder::LittleEndian;

use crate::decode::{Decode, DecodeError};
use crate::encode::Encode;
use crate::mysql::io::{BufExt, BufMutExt};
use crate::mysql::protocol::TypeId;
use crate::mysql::types::MySqlTypeInfo;
use crate::mysql::MySql;
use crate::types::Type;

impl Type<[u8]> for MySql {
    fn type_info() -> MySqlTypeInfo {
        MySqlTypeInfo {
            id: TypeId::TEXT,
            is_binary: true,
            is_unsigned: false,
            char_set: 63, // binary
        }
    }
}

impl Type<Vec<u8>> for MySql {
    fn type_info() -> MySqlTypeInfo {
        <Self as Type<[u8]>>::type_info()
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

impl Decode<MySql> for Vec<u8> {
    fn decode(mut buf: &[u8]) -> Result<Self, DecodeError> {
        Ok(buf
            .get_bytes_lenenc::<LittleEndian>()?
            .unwrap_or_default()
            .to_vec())
    }
}
