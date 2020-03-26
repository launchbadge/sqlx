//! Encoding and decoding of Postgres arrays.

use crate::database::Database;
use crate::decode::Decode;
use crate::encode::Encode;
use crate::postgres::database::Postgres;
use crate::postgres::types::raw::{PgArrayDecoder, PgArrayEncoder};
use crate::postgres::PgValue;
use crate::types::Type;

impl<T> Encode<Postgres> for [T]
where
    T: Encode<Postgres>,
    T: Type<Postgres>,
{
    fn encode(&self, buf: &mut Vec<u8>) {
        let mut encoder = PgArrayEncoder::new(buf);

        for item in self {
            encoder.encode(item);
        }

        encoder.finish();
    }
}

impl<T> Encode<Postgres> for Vec<T>
where
    T: Encode<Postgres>,
    T: Type<Postgres>,
{
    fn encode(&self, buf: &mut Vec<u8>) {
        self.as_slice().encode(buf)
    }
}

impl<'de, T> Decode<'de, Postgres> for Vec<T>
where
    T: 'de,
    T: for<'arr> Decode<'arr, Postgres>,
    [T]: Type<Postgres>,
    T: Type<Postgres>,
{
    fn decode(value: PgValue<'de>) -> crate::Result<Self> {
        PgArrayDecoder::<T>::new(value)?.collect()
    }
}

impl<T, DB> Type<DB> for Vec<Option<T>>
where
    DB: Database,
    [T]: Type<DB>,
{
    #[inline]
    fn type_info() -> DB::TypeInfo {
        <[T] as Type<DB>>::type_info()
    }
}

impl<T, DB> Type<DB> for [Option<T>]
where
    DB: Database,
    [T]: Type<DB>,
{
    #[inline]
    fn type_info() -> DB::TypeInfo {
        <[T] as Type<DB>>::type_info()
    }
}
