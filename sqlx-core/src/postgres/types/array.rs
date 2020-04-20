use crate::database::Database;
use crate::decode::Decode;
use crate::encode::{Encode, IsNull};
use crate::postgres::{PgRawBuffer, PgTypeInfo, Postgres};

impl<T> Encode<Postgres> for [T]
where
    T: PgArrayElement,
{
    fn produces() -> PgTypeInfo {
        T::array_type_info()
    }

    fn encode(&self, buf: &mut PgRawBuffer) -> IsNull {
        todo!()
    }
}

impl<T> Encode<Postgres> for Vec<T>
where
    T: PgArrayElement,
{
    fn produces() -> PgTypeInfo {
        T::array_type_info()
    }

    fn encode(&self, buf: &mut PgRawBuffer) -> IsNull {
        <[T] as Encode<Postgres>>::encode(&**self, buf)
    }
}

pub trait PgArrayElement: Encode<Postgres> {
    fn array_type_info() -> PgTypeInfo;
}

// generate impls of PgArrayElement
// declared as: rust type => array type OID

macro_rules! impl_array_elements {
    ($($ty:ident => $id:ident;)+) => {
        $(
            impl PgArrayElement for $ty {
                fn array_type_info() -> PgTypeInfo {
                    PgTypeInfo :: $id
                }
            }
        )+
    }
}

impl_array_elements! {
    bool => BOOL_ARRAY;
    i16 => INT2_ARRAY;
    i32 => INT4_ARRAY;
    i64 => INT8_ARRAY;
}
