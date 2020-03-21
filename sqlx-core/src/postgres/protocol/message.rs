use crate::postgres::database::Postgres;
use std::convert::TryFrom;

#[derive(Debug, Copy, Clone)]
#[repr(u8)]
pub enum Message {
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

impl TryFrom<u8> for Message {
    type Error = crate::Error<Postgres>;

    fn try_from(type_: u8) -> crate::Result<Postgres, Self> {
        // https://www.postgresql.org/docs/12/protocol-message-formats.html
        Ok(match type_ {
            b'E' => Message::ErrorResponse,
            b'N' => Message::NoticeResponse,
            b'D' => Message::DataRow,
            b'S' => Message::ParameterStatus,
            b'Z' => Message::ReadyForQuery,
            b'R' => Message::Authentication,
            b'K' => Message::BackendKeyData,
            b'C' => Message::CommandComplete,
            b'A' => Message::NotificationResponse,
            b'1' => Message::ParseComplete,
            b'2' => Message::BindComplete,
            b'3' => Message::CloseComplete,
            b'n' => Message::NoData,
            b's' => Message::PortalSuspended,
            b't' => Message::ParameterDescription,
            b'T' => Message::RowDescription,
            b'I' => Message::EmptyQueryResponse,

            id => {
                return Err(protocol_err!("unknown message: {:?}", id).into());
            }
        })
    }
}
