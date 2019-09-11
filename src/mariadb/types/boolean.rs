use super::{MariaDb, MariaDbTypeMetadata};
use crate::{
    deserialize::FromSql,
    mariadb::protocol::{FieldType, ParameterFlag},
    serialize::{IsNull, ToSql},
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

impl ToSql<MariaDb> for bool {
    #[inline]
    fn to_sql(self, buf: &mut Vec<u8>) -> IsNull {
        buf.push(self as u8);

        IsNull::No
    }
}

impl FromSql<MariaDb> for bool {
    #[inline]
    fn from_sql(buf: Option<&[u8]>) -> Self {
        // TODO: Handle optionals
        buf.unwrap()[0] != 0
    }
}
