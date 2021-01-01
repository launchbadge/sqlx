use std::borrow::Cow;
use std::error::Error as StdError;
use std::fmt::{self, Display, Formatter};

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug)]
#[non_exhaustive]
pub enum Error {
    Configuration { message: Cow<'static, str>, source: Option<Box<dyn StdError + Send + Sync>> },

    Network(std::io::Error),
}

impl Error {
    #[doc(hidden)]
    pub fn configuration(
        message: impl Into<Cow<'static, str>>,
        source: impl Into<Box<dyn StdError + Send + Sync>>,
    ) -> Self {
        Self::Configuration { message: message.into(), source: Some(source.into()) }
    }

    #[doc(hidden)]
    pub fn configuration_msg(message: impl Into<Cow<'static, str>>) -> Self {
        Self::Configuration { message: message.into(), source: None }
    }
}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::Network(source) => write!(f, "network: {}", source),

            Self::Configuration { message, source: None } => {
                write!(f, "configuration: {}", message)
            }

            Self::Configuration { message, source: Some(source) } => {
                write!(f, "configuration: {}: {}", message, source)
            }
        }
    }
}

impl StdError for Error {
    fn source(&self) -> Option<&(dyn StdError + 'static)> {
        match self {
            Self::Configuration { source: Some(source), .. } => Some(&**source),

            Self::Network(source) => Some(source),

            _ => None,
        }
    }
}

impl From<std::io::Error> for Error {
    fn from(error: std::io::Error) -> Self {
        Error::Network(error)
    }
}

impl From<std::io::ErrorKind> for Error {
    fn from(error: std::io::ErrorKind) -> Self {
        Error::Network(error.into())
    }
}
