use crate::error::{Error, UnexpectedNullError};

pub trait ResultExt<T>: Sized {
    fn try_unwrap_optional(self) -> crate::Result<T>;
}

impl<T> ResultExt<T> for crate::Result<T> {
    fn try_unwrap_optional(self) -> crate::Result<T> {
        self
    }
}

impl<T> ResultExt<T> for crate::Result<Option<T>> {
    fn try_unwrap_optional(self) -> crate::Result<T> {
        self?.ok_or_else(|| UnexpectedNullError.into())
    }
}

impl<T> ResultExt<Option<T>> for crate::Result<T> {
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
