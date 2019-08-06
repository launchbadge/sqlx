use crate::{
    backend::Backend,
    types::{AsSql, SqlType},
};

/// Annotates the result of [ToSql] to differentiate between an empty value and a null value.
pub enum IsNull {
    /// The value was null (and no data was written to the buffer).
    Yes,

    /// The value was not null.
    ///
    /// This does not necessarily mean that any data was written to the buffer.
    No,
}

/// Serializes a single value to be sent to the database.
pub trait ToSql<B, ST>: AsSql<B>
where
    B: Backend,
    ST: SqlType<B>,
{
    fn to_sql(self, buf: &mut Vec<u8>) -> IsNull;
}

impl<B, ST, T> ToSql<B, ST> for Option<T>
where
    B: Backend,
    ST: SqlType<B>,
    T: ToSql<B, ST>,
{
    #[inline]
    fn to_sql(self, _buf: &mut Vec<u8>) -> IsNull {
        IsNull::Yes
    }
}
