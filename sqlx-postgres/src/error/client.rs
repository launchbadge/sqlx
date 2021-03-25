use std::error::Error as StdError;
use std::fmt::{self, Display, Formatter};
use std::str::Utf8Error;

use sqlx_core::{ClientError, Error};

use crate::protocol::backend::BackendMessageType;

#[derive(Debug)]
#[non_exhaustive]
pub enum PgClientError {
    // attempting to interpret data from postgres as UTF-8, when it should
    // be UTF-8, but for some reason (data corruption?) it is not
    NotUtf8(Utf8Error),
    UnknownAuthenticationMethod(u32),
    UnknownMessageType(u8),
    UnknownTransactionStatus(u8),
    UnknownValueFormat(i16),
    UnexpectedMessageType { ty: u8, context: &'static str },
}

impl Display for PgClientError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::NotUtf8(source) => write!(f, "{}", source),

            Self::UnknownAuthenticationMethod(method) => {
                write!(f, "unknown authentication method: {}", method)
            }

            Self::UnknownTransactionStatus(status) => {
                write!(f, "in ReadyForQuery, unknown transaction status: {}", status)
            }

            Self::UnknownValueFormat(format) => {
                write!(f, "unknown value format: {}", format)
            }

            Self::UnknownMessageType(ty) => {
                write!(f, "unknown protocol message type: '{}' ({})", *ty as char, *ty)
            }

            Self::UnexpectedMessageType { ty, context } => {
                write!(f, "unexpected message {:?} '{}' while {}", ty, (*ty as u8 as char), context)
            }
        }
    }
}

impl StdError for PgClientError {}

impl ClientError for PgClientError {}

impl From<PgClientError> for Error {
    fn from(err: PgClientError) -> Error {
        Error::client(err)
    }
}
