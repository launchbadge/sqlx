use std::borrow::Cow;
use std::error::Error as StdError;
use std::fmt::{self, Display, Formatter};

mod database;

pub use database::DatabaseError;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug)]
#[non_exhaustive]
pub enum Error {
    Configuration { message: Cow<'static, str>, source: Option<Box<dyn StdError + Send + Sync>> },

    Connect(Box<dyn DatabaseError>),

    Network(std::io::Error),
}

impl Error {
    #[doc(hidden)]
    pub fn connect<E>(error: E) -> Self
    where
        E: DatabaseError,
    {
        Self::Connect(Box::new(error))
    }

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
            Self::Network(source) => write!(f, "{}", source),

            Self::Connect(source) => write!(f, "{}", source),

            Self::Configuration { message, source: None } => {
                write!(f, "{}", message)
            }

            Self::Configuration { message, source: Some(source) } => {
                write!(f, "{}: {}", message, source)
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
        Self::Network(error)
    }
}

impl From<std::io::ErrorKind> for Error {
    fn from(error: std::io::ErrorKind) -> Self {
        Self::Network(error.into())
    }
}
