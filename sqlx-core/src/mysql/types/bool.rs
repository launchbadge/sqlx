use std::convert::TryInto;

use crate::decode::Decode;
use crate::encode::Encode;
use crate::mysql::protocol::TypeId;
use crate::mysql::types::MySqlTypeInfo;
use crate::mysql::{MySql, MySqlValue};
use crate::types::Type;

impl Type<MySql> for bool {
    fn type_info() -> MySqlTypeInfo {
        MySqlTypeInfo::new(TypeId::TINY_INT)
    }
}

impl Encode<MySql> for bool {
    fn encode(&self, buf: &mut Vec<u8>) {
        buf.push(*self as u8);
    }
}

impl<'de> Decode<'de, MySql> for bool {
    fn decode(value: Option<MySqlValue<'de>>) -> crate::Result<MySql, Self> {
        match value.try_into()? {
            MySqlValue::Binary(buf) => Ok(buf.get(0).map(|&b| b != 0).unwrap_or_default()),

            MySqlValue::Text(b"0") => Ok(false),

            MySqlValue::Text(b"1") => Ok(true),

            MySqlValue::Text(s) => Err(crate::Error::Decode(
                format!("unexpected value {:?} for boolean", s).into(),
            )),
        }
    }
}
