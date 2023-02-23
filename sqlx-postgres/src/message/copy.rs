use crate::error::Result;
use crate::io::{BufExt, BufMutExt, Decode, Encode};
use sqlx_core::bytes::{Buf, BufMut, Bytes};
use std::ops::Deref;

/// The same structure is sent for both `CopyInResponse` and `CopyOutResponse`
pub struct CopyResponse {
    pub format: i8,
    pub num_columns: i16,
    pub format_codes: Vec<i16>,
}

pub struct CopyData<B>(pub B);

pub struct CopyFail {
    pub message: String,
}

pub struct CopyDone;

impl Decode<'_> for CopyResponse {
    fn decode_with(mut buf: Bytes, _: ()) -> Result<Self> {
        let format = buf.get_i8();
        let num_columns = buf.get_i16();

        let format_codes = (0..num_columns).map(|_| buf.get_i16()).collect();

        Ok(CopyResponse {
            format,
            num_columns,
            format_codes,
        })
    }
}

impl Decode<'_> for CopyData<Bytes> {
    fn decode_with(buf: Bytes, _: ()) -> Result<Self> {
        // well.. that was easy
        Ok(CopyData(buf))
    }
}

impl<B: Deref<Target = [u8]>> Encode<'_> for CopyData<B> {
    fn encode_with(&self, buf: &mut Vec<u8>, _context: ()) {
        buf.push(b'd');
        buf.put_u32(self.0.len() as u32 + 4);
        buf.extend_from_slice(&self.0);
    }
}

impl Decode<'_> for CopyFail {
    fn decode_with(mut buf: Bytes, _: ()) -> Result<Self> {
        Ok(CopyFail {
            message: buf.get_str_nul()?,
        })
    }
}

impl Encode<'_> for CopyFail {
    fn encode_with(&self, buf: &mut Vec<u8>, _: ()) {
        let len = 4 + self.message.len() + 1;

        buf.push(b'f'); // to pay respects
        buf.put_u32(len as u32);
        buf.put_str_nul(&self.message);
    }
}

impl CopyFail {
    pub fn new(msg: impl Into<String>) -> CopyFail {
        CopyFail {
            message: msg.into(),
        }
    }
}

impl Decode<'_> for CopyDone {
    fn decode_with(buf: Bytes, _: ()) -> Result<Self> {
        if buf.is_empty() {
            Ok(CopyDone)
        } else {
            Err(err_protocol!(
                "expected no data for CopyDone, got: {:?}",
                buf
            ))
        }
    }
}

impl Encode<'_> for CopyDone {
    fn encode_with(&self, buf: &mut Vec<u8>, _: ()) {
        buf.reserve(4);
        buf.push(b'c');
        buf.put_u32(4);
    }
}
