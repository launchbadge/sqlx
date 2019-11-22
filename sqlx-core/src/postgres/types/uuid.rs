use uuid::Uuid;

use super::{Postgres, PostgresTypeFormat, PostgresTypeMetadata};
use crate::{
    deserialize::FromSql,
    serialize::{IsNull, ToSql},
    types::HasSqlType,
};

impl HasSqlType<Uuid> for Postgres {
    fn metadata() -> PostgresTypeMetadata {
        PostgresTypeMetadata {
            format: PostgresTypeFormat::Binary,
            oid: 2950,
            array_oid: 2951,
        }
    }
}

impl ToSql<Postgres> for Uuid {
    #[inline]
    fn to_sql(&self, buf: &mut Vec<u8>) -> IsNull {
        buf.extend_from_slice(self.as_bytes());

        IsNull::No
    }
}

impl FromSql<Postgres> for Uuid {
    #[inline]
    fn from_sql(buf: Option<&[u8]>) -> Self {
        // TODO: Handle optionals, error
        Uuid::from_slice(buf.unwrap()).unwrap()
    }
}
