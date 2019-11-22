use super::{MariaDb, MariaDbTypeMetadata};
use crate::{
    decode::Decode,
    encode::{Encode, IsNull},
    mariadb::protocol::{FieldType, ParameterFlag},
    types::HasSqlType,
};

impl HasSqlType<bool> for MariaDb {
    fn metadata() -> MariaDbTypeMetadata {
        MariaDbTypeMetadata {
            // MYSQL_TYPE_TINY
            field_type: FieldType(1),
            param_flag: ParameterFlag::empty(),
        }
    }
}

impl Encode<MariaDb> for bool {
    #[inline]
    fn encode(&self, buf: &mut Vec<u8>) -> IsNull {
        buf.push(*self as u8);

        IsNull::No
    }
}

impl Decode<MariaDb> for bool {
    #[inline]
    fn decode(buf: Option<&[u8]>) -> Self {
        // TODO: Handle optionals
        buf.unwrap()[0] != 0
    }
}
