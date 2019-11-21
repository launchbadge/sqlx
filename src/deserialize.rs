//! Types and traits related to deserializing values from the database.
use crate::{backend::Backend, types::HasSqlType};

// TODO: Allow from_sql to return an error (that can be unified)

pub trait FromSql<DB: Backend> {
    fn from_sql(raw: Option<&[u8]>) -> Self;
}

impl<T, DB> FromSql<DB> for Option<T>
where
    DB: Backend + HasSqlType<T>,
    T: FromSql<DB>,
{
    fn from_sql(raw: Option<&[u8]>) -> Self {
        Some(T::from_sql(Some(raw?)))
    }
}
