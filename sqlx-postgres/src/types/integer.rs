use crate::{PgTypeId, PgTypeInfo, Postgres};
use sqlx_core::{error::BoxStdError, to_value::ToValue, type_info::TypeInfo};

impl ToValue<Postgres> for i32 {
    fn accepts(&self, ty: &PgTypeInfo) -> bool {
        matches!(*ty, PgTypeInfo::INT4)
    }

    fn produces(&self) -> PgTypeId<'static> {
        PgTypeInfo::INT4.id()
    }

    fn to_value(&self, ty: &PgTypeInfo, buf: &mut Vec<u8>) -> Result<(), BoxStdError> {
        buf.extend(&self.to_be_bytes());

        Ok(())
    }
}
