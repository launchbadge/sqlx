use crate::{Decode, Encode, NoticeResponse, ReadyForQuery};
use byteorder::{BigEndian, ReadBytesExt};
use bytes::Bytes;
use std::io::{self, Cursor};

#[derive(Debug)]
pub enum Message {
    ReadyForQuery(ReadyForQuery),
    NoticeResponse(NoticeResponse),
}

impl Encode for Message {
    fn size_hint(&self) -> usize {
        match self {
            Message::ReadyForQuery(body) => body.size_hint(),
            Message::NoticeResponse(body) => body.size_hint(),
        }
    }

    fn encode(&self, buf: &mut Vec<u8>) -> io::Result<()> {
        match self {
            Message::ReadyForQuery(body) => body.encode(buf),
            Message::NoticeResponse(body) => body.encode(buf),
        }
    }
}

impl Decode for Message {
    fn decode(b: Bytes) -> io::Result<Self>
    where
        Self: Sized,
    {
        let mut buf = Cursor::new(&b);

        let token = buf.read_u8()?;
        let len = buf.read_u32::<BigEndian>()? as usize;
        let pos = buf.position() as usize;

        // `len` includes the size of the length u32
        let b = b.slice(pos, pos + len - 4);

        Ok(match token {
            // FIXME: These tokens are duplicated here and in the respective encode functions
            b'N' => Message::NoticeResponse(NoticeResponse::decode(b)?),
            b'Z' => Message::ReadyForQuery(ReadyForQuery::decode(b)?),

            _ => unimplemented!("decode not implemented for token: {}", token as char),
        })
    }
}
