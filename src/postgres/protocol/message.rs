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
    Authentication(Box<Authentication>),
    ParameterStatus(Box<ParameterStatus>),
    BackendKeyData(BackendKeyData),
    ReadyForQuery(ReadyForQuery),
    CommandComplete(CommandComplete),
    RowDescription(Box<RowDescription>),
    DataRow(Box<DataRow>),
    Response(Box<Response>),
    NotificationResponse(Box<NotificationResponse>),
    ParseComplete,
    BindComplete,
    CloseComplete,
    NoData,
    PortalSuspended,
    ParameterDescription(Box<ParameterDescription>),
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
            return Err(io::ErrorKind::InvalidData.into());
        }

        // FIXME: What happens if len(u32) < len(usize) ?
        let len = BigEndian::read_u32(&src[1..5]) as usize;

        if src.len() >= (len + 1) {
            let window = &src[5..=len];

            let message = match token {
                b'N' | b'E' => Message::Response(Box::new(Response::decode(window))),
                b'D' => Message::DataRow(Box::new(DataRow::decode(window))),
                b'S' => Message::ParameterStatus(Box::new(ParameterStatus::decode(window))),
                b'Z' => Message::ReadyForQuery(ReadyForQuery::decode(window)),
                b'R' => Message::Authentication(Box::new(Authentication::decode(window))),
                b'K' => Message::BackendKeyData(BackendKeyData::decode(window)),
                b'T' => Message::RowDescription(Box::new(RowDescription::decode(window))),
                b'C' => Message::CommandComplete(CommandComplete::decode(window)),
                b'A' => {
                    Message::NotificationResponse(Box::new(NotificationResponse::decode(window)))
                }
                b'1' => Message::ParseComplete,
                b'2' => Message::BindComplete,
                b'3' => Message::CloseComplete,
                b'n' => Message::NoData,
                b's' => Message::PortalSuspended,
                b't' => {
                    Message::ParameterDescription(Box::new(ParameterDescription::decode(window)))
                }

                _ => unimplemented!("decode not implemented for token: {}", token as char),
            };

            src.advance(len + 1);

            Ok(Some(message))
        } else {
            // We don't have enough in the stream yet
            Ok(None)
        }
    }
}
