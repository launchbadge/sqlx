use crate::{backend::Backend, types::HasSqlType};

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
pub trait ToSql<A, DB: Backend> {
    fn to_sql(self, buf: &mut Vec<u8>) -> IsNull;
}

impl<T, ST, DB> ToSql<ST, DB> for Option<T>
where
    DB: Backend + HasSqlType<ST>,
    T: ToSql<ST, DB>,
{
    #[inline]
    fn to_sql(self, buf: &mut Vec<u8>) -> IsNull {
        if let Some(self_) = self {
            self_.to_sql(buf)
        } else {
            IsNull::Yes
        }
    }
}
