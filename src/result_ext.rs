use crate::error::{Error, UnexpectedNullError};
use sqlx_core::database::Database;

pub trait ResultExt<DB, T>: Sized
where
    DB: Database,
{
    fn try_unwrap_optional(self) -> crate::Result<T>;
}

impl<DB, T> ResultExt<DB, T> for crate::Result<T>
where
    DB: Database,
{
    fn try_unwrap_optional(self) -> crate::Result<T> {
        self
    }
}

impl<DB, T> ResultExt<DB, Option<T>> for crate::Result<T>
where
    DB: Database,
{
    fn try_unwrap_optional(self) -> crate::Result<Option<T>> {
        match self {
            Ok(val) => Ok(Some(val)),

            Err(Error::Decode(error)) => {
                if let Some(UnexpectedNullError) = error.downcast_ref() {
                    Ok(None)
                } else {
                    Err(Error::Decode(error))
                }
            }

            Err(e) => Err(e),
        }
    }
}
