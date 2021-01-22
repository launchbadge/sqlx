use bytes::{Buf, Bytes};
use bytestring::ByteString;
use sqlx_core::io::BufExt;
use sqlx_core::io::Deserialize;
use sqlx_core::Result;

#[derive(Debug)]
pub struct Notification {
    pub(crate) process_id: u32,
    pub(crate) channel: ByteString,
    pub(crate) payload: ByteString,
}

impl Deserialize<'_, ()> for Notification {
    fn deserialize_with(mut buf: Bytes, _: ()) -> Result<Self> {
        let process_id = buf.get_u32();

        // UNSAFE: This message will not be read.
        #[allow(unsafe_code)]
        let channel = unsafe { buf.get_str_nul_unchecked()? };

        // UNSAFE: This message will not be read.
        #[allow(unsafe_code)]
        let payload = unsafe { buf.get_str_nul_unchecked()? };

        Ok(Self { process_id, channel, payload })
    }
}
