use byteorder::LittleEndian;

use crate::decode::{Decode, DecodeError};
use crate::encode::Encode;
use crate::mysql::protocol::Type;
use crate::mysql::types::MySqlTypeMetadata;
use crate::mysql::io::{BufMutExt, BufExt};
use crate::mysql::MySql;
use crate::types::HasSqlType;

// TODO: We only have support for BLOB below; we map [u8] to BLOB, as we do not have the size information yet

impl HasSqlType<[u8]> for MySql {
    fn metadata() -> MySqlTypeMetadata {
        MySqlTypeMetadata::new(Type::BLOB)
    }
}

impl HasSqlType<Vec<u8>> for MySql {
    fn metadata() -> MySqlTypeMetadata {
        <Self as HasSqlType<[u8]>>::metadata()
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
        Ok(buf.get_bytes_lenenc::<LittleEndian>()?.unwrap_or_default().to_vec())
    }
}
