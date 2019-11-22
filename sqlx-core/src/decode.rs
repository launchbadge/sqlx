//! Types and traits related to deserializing values from the database.
use crate::{backend::Backend, types::HasSqlType};

// TODO: Allow decode to return an error (that can be unified)

pub trait Decode<DB: Backend> {
    fn decode(raw: Option<&[u8]>) -> Self;
}

impl<T, DB> Decode<DB> for Option<T>
where
    DB: Backend + HasSqlType<T>,
    T: Decode<DB>,
{
    fn decode(raw: Option<&[u8]>) -> Self {
        Some(T::decode(Some(raw?)))
    }
}
