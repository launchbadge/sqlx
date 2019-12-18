use crate::{
    encode::IsNull,
    mysql::{
        protocol::{FieldType, ParameterFlag},
        types::MySqlTypeMetadata,
    },
    Decode, Encode, HasSqlType, MySql,
};

impl HasSqlType<[u8]> for MySql {
    fn metadata() -> MySqlTypeMetadata {
        MySqlTypeMetadata {
            field_type: FieldType::MYSQL_TYPE_BLOB,
            param_flag: ParameterFlag::empty(),
        }
    }
}

impl HasSqlType<Vec<u8>> for MySql {
    fn metadata() -> MySqlTypeMetadata {
        <Self as HasSqlType<[u8]>>::metadata()
    }
}

impl Encode<MySql> for [u8] {
    fn encode(&self, buf: &mut Vec<u8>) -> IsNull {
        buf.extend_from_slice(self);
        IsNull::No
    }
}

impl Encode<MySql> for Vec<u8> {
    fn encode(&self, buf: &mut Vec<u8>) -> IsNull {
        <[u8] as Encode<MySql>>::encode(self, buf)
    }
}

impl Decode<MySql> for Vec<u8> {
    fn decode(raw: Option<&[u8]>) -> Self {
        raw.unwrap().into()
    }
}
