use crate::decode::{Decode, DecodeError};
use crate::encode::Encode;
use crate::mysql::protocol::TypeId;
use crate::mysql::types::MySqlTypeInfo;
use crate::mysql::MySql;
use crate::types::HasSqlType;

impl HasSqlType<f32> for MySql {
    fn type_info() -> MySqlTypeInfo {
        MySqlTypeInfo::new(TypeId::FLOAT)
    }
}

impl Encode<MySql> for f32 {
    fn encode(&self, buf: &mut Vec<u8>) {
        <i32 as Encode<MySql>>::encode(&(self.to_bits() as i32), buf);
    }
}

impl Decode<MySql> for f32 {
    fn decode(buf: &[u8]) -> Result<Self, DecodeError> {
        Ok(f32::from_bits(<i32 as Decode<MySql>>::decode(buf)? as u32))
    }
}

impl HasSqlType<f64> for MySql {
    fn type_info() -> MySqlTypeInfo {
        MySqlTypeInfo::new(TypeId::DOUBLE)
    }
}

impl Encode<MySql> for f64 {
    fn encode(&self, buf: &mut Vec<u8>) {
        <i64 as Encode<MySql>>::encode(&(self.to_bits() as i64), buf);
    }
}

impl Decode<MySql> for f64 {
    fn decode(buf: &[u8]) -> Result<Self, DecodeError> {
        Ok(f64::from_bits(<i64 as Decode<MySql>>::decode(buf)? as u64))
    }
}
