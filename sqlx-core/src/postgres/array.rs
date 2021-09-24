use crate::postgres::PgTypeInfo;

pub trait PgHasArrayType {
    fn array_type_info() -> PgTypeInfo;
    fn array_compatible(ty: &PgTypeInfo) -> bool {
        *ty == Self::array_type_info()
    }
}

impl<T> PgHasArrayType for Option<T>
where
    T: PgHasArrayType,
{
    fn array_type_info() -> PgTypeInfo {
        T::array_type_info()
    }

    fn array_compatible(ty: &PgTypeInfo) -> bool {
        T::array_compatible(ty)
    }
}
