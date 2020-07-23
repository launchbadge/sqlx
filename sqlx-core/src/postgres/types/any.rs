use crate::{
    encode::Encode,
    postgres::{PgArgumentBuffer, PgTypeInfo, Postgres},
    types::Type,
};

/// Specifies the given argument to be untyped, allowing statements with
/// varying types.
///
/// Wrapping a parameter to `PgAny` will not trigger any type checks when
/// preparing the statement.
///
/// `PgAny` is meant to be used when needing to write data without declaring the
/// type in the statement, and cannot be used for reading.
pub struct PgAny<T>(pub T)
where
    T: for<'e> Encode<'e, Postgres>;

impl<T> Encode<'_, Postgres> for PgAny<T>
where
    T: for<'e> Encode<'e, Postgres>,
{
    fn encode_by_ref(&self, buf: &mut PgArgumentBuffer) -> crate::encode::IsNull {
        <T as Encode<'_, Postgres>>::encode_by_ref(&self.0, buf)
    }
}

impl<T> Type<Postgres> for PgAny<T>
where
    T: for<'e> Encode<'e, Postgres>,
{
    fn type_info() -> PgTypeInfo {
        PgTypeInfo::ANY
    }

    fn compatible(_: &PgTypeInfo) -> bool {
        true
    }
}

impl<T> Type<Postgres> for [PgAny<T>]
where
    T: for<'e> Encode<'e, Postgres>,
{
    fn type_info() -> PgTypeInfo {
        PgTypeInfo::ANY
    }

    fn compatible(_: &PgTypeInfo) -> bool {
        true
    }
}

impl<T> Type<Postgres> for Vec<PgAny<T>>
where
    T: for<'e> Encode<'e, Postgres>,
{
    fn type_info() -> PgTypeInfo {
        <[PgAny<T>] as Type<Postgres>>::type_info()
    }

    fn compatible(_: &PgTypeInfo) -> bool {
        true
    }
}
