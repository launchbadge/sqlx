use crate::{Decode, NoticeResponse};
use byteorder::{BigEndian, ReadBytesExt};
use bytes::Bytes;
use std::io::{self, Cursor};

#[derive(Debug)]
#[non_exhaustive]
pub enum Message {
    NoticeResponse(NoticeResponse),
}

impl Decode for Message {
    fn decode(b: Bytes) -> io::Result<Self>
    where Self: Sized {
        let mut buf = Cursor::new(&b);

        let token = buf.read_u8()?;
        let len = buf.read_u32::<BigEndian>()? as usize;
        let pos = buf.position() as usize;

        // `len` includes the size of the length u32
        let b = b.slice(pos, pos + len - 4);

        Ok(match token {
            b'N' => Message::NoticeResponse(NoticeResponse::decode(b)?),

            _ => unimplemented!("decode not implemented for token: {}", token as char),
        })
    }
}
