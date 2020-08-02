use bytes::Bytes;
use sqlx_core::{error::Error, io::Decode};

mod authentication;
mod backend_key_data;
mod command_complete;
// mod data_row;
// mod notice;
// mod notification;
// mod parameter_description;
mod ready_for_query;
// mod row_description;
// mod ssl_request;

pub(crate) use authentication::{Authentication, AuthenticationMd5Password};
pub(crate) use backend_key_data::BackendKeyData;
pub(crate) use command_complete::CommandComplete;
pub(crate) use ready_for_query::{ReadyForQuery, TransactionStatus};

// https://www.postgresql.org/docs/current/protocol-message-formats.html

#[derive(Debug)]
#[repr(u8)]
pub(crate) enum MessageFormat {
    Authentication,
    BackendKeyData,
    BindComplete,
    CloseComplete,
    CommandComplete,
    DataRow,
    EmptyQueryResponse,
    ErrorResponse,
    NoData,
    NoticeResponse,
    NotificationResponse,
    ParameterDescription,
    ParameterStatus,
    ParseComplete,
    PortalSuspended,
    ReadyForQuery,
    RowDescription,
}

#[derive(Debug)]
pub(crate) struct RawMessage {
    pub(crate) format: MessageFormat,
    pub(crate) contents: Bytes,
}

impl RawMessage {
    #[inline]
    pub(crate) fn decode<'de, T>(self) -> Result<T, Error>
    where
        T: Decode<'de>,
    {
        T::decode(self.contents)
    }
}

impl MessageFormat {
    pub(crate) fn try_from_u8(v: u8) -> Result<Self, Error> {
        Ok(match v {
            b'1' => MessageFormat::ParseComplete,
            b'2' => MessageFormat::BindComplete,
            b'3' => MessageFormat::CloseComplete,
            b'C' => MessageFormat::CommandComplete,
            b'D' => MessageFormat::DataRow,
            b'E' => MessageFormat::ErrorResponse,
            b'I' => MessageFormat::EmptyQueryResponse,
            b'A' => MessageFormat::NotificationResponse,
            b'K' => MessageFormat::BackendKeyData,
            b'N' => MessageFormat::NoticeResponse,
            b'R' => MessageFormat::Authentication,
            b'S' => MessageFormat::ParameterStatus,
            b'T' => MessageFormat::RowDescription,
            b'Z' => MessageFormat::ReadyForQuery,
            b'n' => MessageFormat::NoData,
            b's' => MessageFormat::PortalSuspended,
            b't' => MessageFormat::ParameterDescription,

            _ => {
                return Err(Error::Protocol(
                    format!("unknown message type: {:?}", v as char).into(),
                ))
            }
        })
    }
}
