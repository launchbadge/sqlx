use std::convert::TryFrom;

use bytes::{Buf, Bytes};
use sqlx_core::io::Deserialize;
use sqlx_core::Error;
use sqlx_core::Result;

mod authentication;
mod backend_key_data;
mod close;
mod notification;
mod password;
mod ready_for_query;
mod response;
mod startup;
mod terminate;

pub(crate) use authentication::{Authentication, AuthenticationSasl};
pub(crate) use backend_key_data::BackendKeyData;
pub(crate) use close::Close;
pub(crate) use notification::Notification;
pub(crate) use password::Password;
pub(crate) use ready_for_query::ReadyForQuery;
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
    BackendKeyData = b'K',
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
            b'K' => MessageType::BackendKeyData,
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

impl Deserialize<'_, ()> for Message {
    fn deserialize_with(mut buf: Bytes, _: ()) -> Result<Self> {
        let r#type = MessageType::try_from(buf.get_u8())?;
        let size = buf.get_u32() - 4;
        let contents = buf.split_to(size as usize);

        Ok(Message { r#type, contents })
    }
}
