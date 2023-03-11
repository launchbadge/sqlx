use crate::{
    database::{Database, HasArguments, HasValueRef},
    decode::Decode,
    encode::{Encode, IsNull},
    error::BoxDynError,
    types::Type,
};
use std::sync::Arc;

impl<DB> Type<DB> for Arc<str>
where
    DB: Database,
    str: Type<DB>,
{
    fn type_info() -> DB::TypeInfo {
        <str as Type<DB>>::type_info()
    }
}

impl<DB> Decode<'_, DB> for Arc<str>
where
    DB: Database,
    for<'any> &'any str: Decode<'any, DB>,
{
    fn decode(value: <DB as HasValueRef<'_>>::ValueRef) -> Result<Self, BoxDynError> {
        <&'_ str as Decode<'_, DB>>::decode(value).map(Arc::from)
    }
}

impl<'q, DB> Encode<'q, DB> for Arc<str>
where
    DB: Database,
    for<'any> &'any str: Encode<'q, DB>,
{
    fn encode_by_ref(&self, buf: &mut <DB as HasArguments<'q>>::ArgumentBuffer) -> IsNull {
        self.as_ref().encode_by_ref(buf)
    }
}

impl<DB, T> Type<DB> for Arc<[T]>
where
    DB: Database,
    [T]: Type<DB>,
{
    fn type_info() -> DB::TypeInfo {
        <[T] as Type<DB>>::type_info()
    }
}

impl<'q, DB, T> Decode<'q, DB> for Arc<[T]>
where
    Vec<T>: Decode<'q, DB>,
    T: Decode<'q, DB>,
    DB: Database,
{
    fn decode(value: <DB as HasValueRef<'q>>::ValueRef) -> Result<Self, BoxDynError> {
        <Vec<T> as Decode<'q, DB>>::decode(value).map(Arc::from)
    }
}

impl<'q, DB, T> Encode<'q, DB> for Arc<[T]>
where
    for<'any> &'any [T]: Encode<'q, DB>,
    T: Encode<'q, DB>,
    DB: Database,
{
    fn encode_by_ref(&self, buf: &mut <DB as HasArguments<'q>>::ArgumentBuffer) -> IsNull {
        self.as_ref().encode_by_ref(buf)
    }
}
