use sqlx_core::{Null, Type};

use crate::{PgTypeId, PgTypeInfo, Postgres};

impl Type<Postgres> for Null {
    fn type_id() -> PgTypeId
    where
        Self: Sized,
    {
        PgTypeId::Oid(0)
    }

    fn compatible(_: &PgTypeInfo) -> bool
    where
        Self: Sized,
    {
        true
    }
}
