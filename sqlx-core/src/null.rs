use std::ops::Not;

use crate::database::{HasOutput, HasRawValue};
use crate::{decode, encode, Database, Decode, Encode, RawValue, Type};

#[derive(Debug)]
pub struct Null;

impl<Db: Database, T: Type<Db>> Type<Db> for Option<T>
where
    Null: Type<Db>,
{
    fn type_id() -> <Db as Database>::TypeId
    where
        Self: Sized,
    {
        T::type_id()
    }

    fn compatible(ty: &<Db as Database>::TypeInfo) -> bool
    where
        Self: Sized,
    {
        T::compatible(ty)
    }
}

impl<Db: Database, T: Encode<Db>> Encode<Db> for Option<T>
where
    Null: Encode<Db>,
{
    fn encode(
        &self,
        ty: &<Db as Database>::TypeInfo,
        out: &mut <Db as HasOutput<'_>>::Output,
    ) -> encode::Result {
        match self {
            Some(value) => value.encode(ty, out),
            None => Null.encode(ty, out),
        }
    }
}

impl<'r, Db: Database, T: Decode<'r, Db>> Decode<'r, Db> for Option<T>
where
    Null: Decode<'r, Db>,
{
    fn decode(value: <Db as HasRawValue<'r>>::RawValue) -> decode::Result<Self> {
        value.is_null().not().then(|| T::decode(value)).transpose()
    }
}
