use super::{MySql, MariaDbTypeMetadata};
use crate::{
    decode::Decode,
    encode::{Encode, IsNull},
    mysql::protocol::{FieldType, ParameterFlag},
    types::HasSqlType,
};

impl HasSqlType<bool> for MySql {
    fn metadata() -> MariaDbTypeMetadata {
        MariaDbTypeMetadata {
            // MYSQL_TYPE_TINY
            field_type: FieldType::MYSQL_TYPE_TINY,
            param_flag: ParameterFlag::empty(),
        }
    }
}

impl Encode<MySql> for bool {
    #[inline]
    fn encode(&self, buf: &mut Vec<u8>) -> IsNull {
        buf.push(*self as u8);

        IsNull::No
    }
}

impl Decode<MySql> for bool {
    #[inline]
    fn decode(buf: Option<&[u8]>) -> Self {
        // TODO: Handle optionals
        buf.unwrap()[0] != 0
    }
}
