use super::{
    Authentication, BackendKeyData, CommandComplete, DataRow, Decode, NotificationResponse,
    ParameterDescription, ParameterStatus, ReadyForQuery, Response,
};
use byteorder::{BigEndian, ByteOrder};
use bytes::BytesMut;
use std::io;

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
}
