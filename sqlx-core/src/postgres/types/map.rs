use crate::types::Type;
use crate::postgres::types::array_compatible;
use crate::postgres::{PgArgumentBuffer, PgTypeInfo, PgValueFormat, PgValueRef, Postgres};
use serde_json::{Map, Value};

impl Type<Postgres> for Map<String, Value> {
    fn type_info() -> PgTypeInfo {
        PgTypeInfo::JSONB
    }

    fn compatible(ty: &PgTypeInfo) -> bool {
        *ty == PgTypeInfo::JSON || *ty == PgTypeInfo::JSONB
    }
}

impl Type<Postgres> for [Map<String, Value>] {
    fn type_info() -> PgTypeInfo {
        PgTypeInfo::JSONB_ARRAY
    }

    fn compatible(ty: &PgTypeInfo) -> bool {
        array_compatible::<Map<String, Value>>(ty)
    }
}

impl Type<Postgres> for Vec<Map<String, Value>> {
    fn type_info() -> PgTypeInfo {
        <[Map<String, Value>] as Type<Postgres>>::type_info()
    }

    fn compatible(ty: &PgTypeInfo) -> bool {
        <[Map<String, Value>] as Type<Postgres>>::compatible(ty)
    }
}
