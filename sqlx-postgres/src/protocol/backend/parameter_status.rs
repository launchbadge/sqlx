use bytes::Bytes;
use bytestring::ByteString;
use sqlx_core::io::{BufExt, Deserialize};
use sqlx_core::Result;

#[derive(Debug)]
pub(crate) struct ParameterStatus {
    pub(crate) name: ByteString,
    pub(crate) value: ByteString,
}

impl Deserialize<'_> for ParameterStatus {
    fn deserialize_with(mut buf: Bytes, _: ()) -> Result<Self> {
        let name = buf.get_str_nul()?;
        let value = buf.get_str_nul()?;

        Ok(Self { name, value })
    }
}
