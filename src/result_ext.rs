use crate::decode::DecodeError;
use crate::Error;

pub trait ResultExt<T>: Sized {
    fn try_unwrap_optional(self) -> crate::Result<T>;
}

impl<T> ResultExt<T> for crate::Result<T> {
    fn try_unwrap_optional(self) -> crate::Result<T> {
        self
    }
}

impl<T> ResultExt<Option<T>> for crate::Result<T> {
    fn try_unwrap_optional(self) -> crate::Result<Option<T>> {
        match self {
            Ok(val) => Ok(Some(val)),
            Err(Error::Decode(DecodeError::UnexpectedNull)) => Ok(None),
            Err(e) => Err(e),
        }
    }
}
