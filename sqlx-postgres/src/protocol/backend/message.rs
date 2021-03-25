use std::convert::TryFrom;
use std::fmt::Debug;

use bytes::Bytes;
use sqlx_core::io::Deserialize;
use sqlx_core::{Error, Result};

use crate::PgClientError;

/// Type of the *incoming* message.
///
/// Postgres does use the same message format for client and server messages but we are only
/// interested in messages from the backend.
///
#[derive(Debug, Clone, Copy)]
#[repr(u8)]
pub(crate) enum BackendMessageType {
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
    CopyInResponse = b'G',
    CopyOutResponse = b'H',
    CopyBothResponse = b'W',
    CopyData = b'd',
    CopyDone = b'c',
}

impl TryFrom<u8> for BackendMessageType {
    type Error = Error;

    fn try_from(ty: u8) -> Result<Self> {
        Ok(match ty {
            b'1' => Self::ParseComplete,
            b'2' => Self::BindComplete,
            b'3' => Self::CloseComplete,
            b'C' => Self::CommandComplete,
            b'D' => Self::DataRow,
            b'E' => Self::ErrorResponse,
            b'I' => Self::EmptyQueryResponse,
            b'A' => Self::NotificationResponse,
            b'K' => Self::BackendKeyData,
            b'N' => Self::NoticeResponse,
            b'R' => Self::Authentication,
            b'S' => Self::ParameterStatus,
            b'T' => Self::RowDescription,
            b'Z' => Self::ReadyForQuery,
            b'n' => Self::NoData,
            b's' => Self::PortalSuspended,
            b't' => Self::ParameterDescription,
            b'G' => Self::CopyInResponse,
            b'H' => Self::CopyOutResponse,
            b'W' => Self::CopyBothResponse,
            b'd' => Self::CopyData,
            b'c' => Self::CopyDone,

            _ => {
                return Err(PgClientError::UnknownMessageType(ty).into());
            }
        })
    }
}

#[derive(Debug)]
pub(crate) struct BackendMessage {
    pub(crate) ty: BackendMessageType,
    pub(crate) contents: Bytes,
}

impl BackendMessage {
    #[inline]
    pub(crate) fn deserialize<'de, T>(self) -> Result<T>
    where
        T: Deserialize<'de> + Debug,
    {
        let packet = T::deserialize(self.contents)?;

        log::trace!("read  > {:?}", packet);

        Ok(packet)
    }
}
