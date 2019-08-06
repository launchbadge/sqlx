use crate::{
    backend::Backend,
    types::{AsSql, SqlType},
};

// TODO: Allow from_sql to return an error (that can be unified)
// TODO: Consider using a RawValue wrapper type instead of exposing raw bytes (as different back-ends may want to expose different data here.. maybe?)

pub trait FromSql<B, ST>: AsSql<B>
where
    B: Backend,
    ST: SqlType<B>,
{
    fn from_sql(raw: Option<&[u8]>) -> Self;
}

impl<B, ST, T> FromSql<B, ST> for Option<T>
where
    B: Backend,
    ST: SqlType<B>,
    T: FromSql<B, ST>,
{
    #[inline]
    fn from_sql(raw: Option<&[u8]>) -> Self {
        Some(T::from_sql(Some(raw?)))
    }
}
