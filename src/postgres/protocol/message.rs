use super::{
    Authentication, BackendKeyData, CommandComplete, DataRow, NotificationResponse,
    ParameterDescription, ParameterStatus, ReadyForQuery, Response,
};
use crate::postgres::protocol::row_description::RowDescription;

#[derive(Debug)]
#[repr(u8)]
pub enum Message {
    Authentication(Box<Authentication>),
    ParameterStatus(Box<ParameterStatus>),
    BackendKeyData(BackendKeyData),
    ReadyForQuery(ReadyForQuery),
    CommandComplete(CommandComplete),
    DataRow(DataRow),
    Response(Box<Response>),
    NotificationResponse(Box<NotificationResponse>),
    ParseComplete,
    BindComplete,
    CloseComplete,
    NoData,
    PortalSuspended,
    ParameterDescription(Box<ParameterDescription>),
    RowDescription(Box<RowDescription>)
}
