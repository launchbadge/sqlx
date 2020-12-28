use std::error::Error as StdError;
use std::fmt::{self, Display, Formatter};

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug)]
#[non_exhaustive]
pub enum Error {
    InvalidConnectionUrl(url::ParseError),
}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidConnectionUrl(source) => write!(f, "invalid connection url: {}", source),
        }
    }
}

impl StdError for Error {
    fn source(&self) -> Option<&(dyn StdError + 'static)> {
        match self {
            Self::InvalidConnectionUrl(source) => Some(source),
        }
    }
}
