use std::convert::TryFrom;

use bytes::Bytes;
use sqlx_core::io::Deserialize;
use sqlx_core::Error;
use sqlx_core::Result;

mod close;
mod notification;
mod response;
mod startup;
mod terminate;

pub(crate) use close::Close;
pub(crate) use notification::Notification;
pub(crate) use response::{Notice, PgSeverity};
pub(crate) use startup::Startup;
pub(crate) use terminate::Terminate;

#[derive(Debug, Clone, Copy, PartialOrd, PartialEq, Eq)]
#[repr(u8)]
pub enum MessageType {
    ParseComplete = b'1',
    BindComplete = b'2',
    CloseComplete = b'3',
    CommandComplete = b'C',
    DataRow = b'D',
    ErrorResponse = b'E',
    EmptyQueryResponse = b'I',
    NotificationResponse = b'A',
    KeyData = b'K',
    NoticeResponse = b'N',
    Authentication = b'R',
    ParameterStatus = b'S',
    RowDescription = b'T',
    ReadyForQuery = b'Z',
    NoData = b'n',
    PortalSuspended = b's',
    ParameterDescription = b't',
}

#[derive(Debug)]
pub struct Message {
    pub r#type: MessageType,
    pub contents: Bytes,
}

impl Message {
    #[inline]
    pub fn decode<'de, T>(self) -> Result<T>
    where
        T: Deserialize<'de, ()>,
    {
        T::deserialize_with(self.contents, ())
    }
}

impl TryFrom<u8> for MessageType {
    type Error = Error;

    fn try_from(v: u8) -> Result<Self> {
        // https://www.postgresql.org/docs/current/protocol-message-formats.html

        Ok(match v {
            b'1' => MessageType::ParseComplete,
            b'2' => MessageType::BindComplete,
            b'3' => MessageType::CloseComplete,
            b'C' => MessageType::CommandComplete,
            b'D' => MessageType::DataRow,
            b'E' => MessageType::ErrorResponse,
            b'I' => MessageType::EmptyQueryResponse,
            b'A' => MessageType::NotificationResponse,
            b'K' => MessageType::KeyData,
            b'N' => MessageType::NoticeResponse,
            b'R' => MessageType::Authentication,
            b'S' => MessageType::ParameterStatus,
            b'T' => MessageType::RowDescription,
            b'Z' => MessageType::ReadyForQuery,
            b'n' => MessageType::NoData,
            b's' => MessageType::PortalSuspended,
            b't' => MessageType::ParameterDescription,

            _ => {
                return Err(Error::configuration_msg(format!(
                    "unknown message type: {:?}",
                    v as char
                )));
            }
        })
    }
}
