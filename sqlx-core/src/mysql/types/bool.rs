use crate::decode::{Decode, DecodeError};
use crate::encode::Encode;
use crate::mysql::protocol::Type;
use crate::mysql::types::MySqlTypeMetadata;
use crate::mysql::MySql;
use crate::types::HasSqlType;

impl HasSqlType<bool> for MySql {
    fn metadata() -> MySqlTypeMetadata {
        MySqlTypeMetadata::new(Type::TINY)
    }
}

impl Encode<MySql> for bool {
    fn encode(&self, buf: &mut Vec<u8>) {
        buf.push(*self as u8);
    }
}

impl Decode<MySql> for bool {
    fn decode(buf: &[u8]) -> Result<Self, DecodeError> {
        // FIXME: Return an error if the buffer size is not (at least) 1
        Ok(buf[0] != 0)
    }
}
