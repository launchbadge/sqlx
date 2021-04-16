use std::error::Error as StdError;
use std::fmt::{self, Display, Formatter};

use crate::protocol::AuthPlugin;
use sqlx_core::{ClientError, Error};

#[derive(Debug)]
#[non_exhaustive]
pub enum MySqlClientError {
    UnknownAuthPlugin(String),
    AuthPlugin { plugin: &'static str, source: Box<dyn StdError + 'static + Send + Sync> },
    EmptyPacket { context: &'static str },
    UnexpectedPacketSize { expected: usize, actual: usize },
}

impl MySqlClientError {
    pub(crate) fn auth_plugin(
        plugin: &impl AuthPlugin,
        source: impl Into<Box<dyn StdError + Send + Sync>>,
    ) -> Self {
        Self::AuthPlugin { plugin: plugin.name(), source: source.into() }
    }
}

impl Display for MySqlClientError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnknownAuthPlugin(name) => write!(f, "unknown authentication plugin: {}", name),

            Self::AuthPlugin { plugin, source } => {
                write!(f, "authentication plugin '{}' reported error: {}", plugin, source)
            }

            Self::EmptyPacket { context } => write!(f, "received no bytes for {}", context),

            Self::UnexpectedPacketSize { actual, expected } => {
                write!(f, "received {} bytes for packet but expecting {} bytes", actual, expected)
            }
        }
    }
}

impl StdError for MySqlClientError {}

impl ClientError for MySqlClientError {}

impl From<MySqlClientError> for Error {
    fn from(err: MySqlClientError) -> Self {
        Self::client(err)
    }
}
