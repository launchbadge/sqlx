use crate::{
    Authentication, BackendKeyData, CommandComplete, DataRow, Decode, NotificationResponse,
    ParameterStatus, ReadyForQuery, Response, RowDescription,
};
use byteorder::{BigEndian, ByteOrder};
use bytes::BytesMut;
use std::io;

#[derive(Debug)]
pub enum Message {
    Authentication(Authentication),
    ParameterStatus(ParameterStatus),
    BackendKeyData(BackendKeyData),
    ReadyForQuery(ReadyForQuery),
    CommandComplete(CommandComplete),
    RowDescription(RowDescription),
    DataRow(DataRow),
    Response(Response),
    NotificationResponse(NotificationResponse),
    ParseComplete,
    BindComplete,
    NoData,
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

        let token = src[0];
        if token == 0 {
            // FIXME: Handle end-of-stream
            return Err(io::ErrorKind::InvalidData)?;
        }

        // FIXME: What happens if len(u32) < len(usize) ?
        let len = BigEndian::read_u32(&src[1..5]) as usize;

        if src.len() < len {
            // We don't have enough in the stream yet
            return Ok(None);
        }

        let src = src.split_to(len + 1).freeze().slice_from(5);

        log::trace!("recv {:?}", src);

        let message = match token {
            b'N' | b'E' => Message::Response(Response::decode(src)?),
            b'S' => Message::ParameterStatus(ParameterStatus::decode(src)?),
            b'Z' => Message::ReadyForQuery(ReadyForQuery::decode(src)?),
            b'R' => Message::Authentication(Authentication::decode(src)?),
            b'K' => Message::BackendKeyData(BackendKeyData::decode(src)?),
            b'T' => Message::RowDescription(RowDescription::decode(src)?),
            b'D' => Message::DataRow(DataRow::decode(src)?),
            b'C' => Message::CommandComplete(CommandComplete::decode(src)?),
            b'A' => Message::NotificationResponse(NotificationResponse::decode(src)?),
            b'1' => Message::ParseComplete,
            b'2' => Message::BindComplete,
            b'n' => Message::NoData,

            _ => unimplemented!("decode not implemented for token: {}", token as char),
        };

        log::trace!("decode {:?}", message);

        Ok(Some(message))
    }
}
