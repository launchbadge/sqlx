use crate::{backend::Backend, types::HasSqlType};

// TODO: Allow from_sql to return an error (that can be unified)
// TODO: Consider using a RawValue wrapper type inUead of exposing raw bytes (as different back-ends may want to expose different data here.. maybe?)

pub trait FromSql<A, DB: Backend> {
    fn from_sql(raw: Option<&[u8]>) -> Self;
}

impl<T, ST, DB> FromSql<ST, DB> for Option<T>
where
    DB: Backend + HasSqlType<ST>,
    T: FromSql<ST, DB>,
{
    #[inline]
    fn from_sql(raw: Option<&[u8]>) -> Self {
        Some(T::from_sql(Some(raw?)))
    }
}
