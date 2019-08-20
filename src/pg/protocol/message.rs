use super::{
    Authentication, BackendKeyData, CommandComplete, DataRow, Decode, NotificationResponse,
    ParameterDescription, ParameterStatus, ReadyForQuery, Response, RowDescription,
};
use byteorder::{BigEndian, ByteOrder};
use bytes::BytesMut;
use std::io;

#[derive(Debug)]
#[repr(u8)]
pub enum Message {
    Authentication(Authentication),
    ParameterStatus(ParameterStatus),
    BackendKeyData(BackendKeyData),
    ReadyForQuery(ReadyForQuery),
    CommandComplete(CommandComplete),
    RowDescription(RowDescription),
    DataRow(DataRow),
    Response(Box<Response>),
    NotificationResponse(NotificationResponse),
    ParseComplete,
    BindComplete,
    CloseComplete,
    NoData,
    PortalSuspended,
    ParameterDescription(ParameterDescription),
}

impl Message {
    // FIXME: `Message::decode` shares the name of the remaining message type `::decode` despite being very
    //        different
    pub fn decode(src: &mut BytesMut) -> io::Result<Option<Self>>
    where
        Self: Sized,
    {
        if src.len() < 5 {
            // No message is less than 5 bytes
            return Ok(None);
        }

        log::trace!("[postgres] [decode] {:?}", bytes::Bytes::from(src.as_ref()));

        let token = src[0];
        if token == 0 {
            // FIXME: Handle end-of-stream
            return Err(io::ErrorKind::InvalidData)?;
        }

        // FIXME: What happens if len(u32) < len(usize) ?
        let len = BigEndian::read_u32(&src[1..5]) as usize;

        if src.len() < (len + 1) {
            // We don't have enough in the stream yet
            return Ok(None);
        }

        let src_ = &src.as_ref()[5..(len + 1)];

        let message = match token {
            b'N' | b'E' => Message::Response(Box::new(Response::decode(src_)?)),
            b'D' => Message::DataRow(DataRow::decode2(src_)),
            b'S' => Message::ParameterStatus(ParameterStatus::decode(src_)?),
            b'Z' => Message::ReadyForQuery(ReadyForQuery::decode(src_)?),
            b'R' => Message::Authentication(Authentication::decode(src_)?),
            b'K' => Message::BackendKeyData(BackendKeyData::decode2(src_)),
            b'T' => Message::RowDescription(RowDescription::decode(src_)?),
            b'C' => Message::CommandComplete(CommandComplete::decode2(src_)),
            b'A' => Message::NotificationResponse(NotificationResponse::decode(src_)?),
            b'1' => Message::ParseComplete,
            b'2' => Message::BindComplete,
            b'3' => Message::CloseComplete,
            b'n' => Message::NoData,
            b's' => Message::PortalSuspended,
            b't' => Message::ParameterDescription(ParameterDescription::decode2(src_)),
            _ => unimplemented!("decode not implemented for token: {}", token as char),
        };

        src.advance(len + 1);

        Ok(Some(message))
    }
}
