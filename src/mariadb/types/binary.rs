use crate::{HasSqlType, MariaDb, ToSql, FromSql};
use crate::mariadb::types::MariaDbTypeMetadata;
use crate::mariadb::protocol::{ParameterFlag, FieldType};
use crate::serialize::IsNull;

impl HasSqlType<[u8]> for MariaDb {
    fn metadata() -> Self::Metadata {
        MariaDbTypeMetadata {
            field_type: FieldType::MYSQL_TYPE_BLOB,
            param_flag: ParameterFlag::empty(),
        }
    }
}

impl HasSqlType<Vec<u8>> for MariaDb {
    fn metadata() -> Self::Metadata {
        <Self as HasSqlType<[u8]>>::metadata()
    }
}

impl ToSql<MariaDb> for [u8] {
    fn to_sql(&self, buf: &mut Vec<u8>) -> IsNull {
        buf.extend_from_slice(self);
        IsNull::No
    }
}

impl ToSql<MariaDb> for Vec<u8> {
    fn to_sql(&self, buf: &mut Vec<u8>) -> IsNull {
        <[u8] as ToSql<MariaDb>>::to_sql(self, buf)
    }
}

impl FromSql<MariaDb> for Vec<u8> {
    fn from_sql(raw: Option<&[u8]>) -> Self {
        raw.unwrap().into()
    }
}
