use crate::error::Error;
use bytes::Bytes;

/// An object that can be decoded from a byte buffer.
/// Context is optional metadata that is required to decode this object.
pub trait Decode<'de, Context = ()>: Sized {
    #[inline]
    fn decode(buf: Bytes) -> Result<Self, Error>
    where
        Self: Decode<'de, ()>,
    {
        Self::decode_with(buf, ())
    }

    fn decode_with(buf: Bytes, context: Context) -> Result<Self, Error>;
}

impl<C> Decode<'_, C> for Bytes {
    #[inline]
    fn decode_with(buf: Bytes, _: C) -> Result<Self, Error> {
        Ok(buf)
    }
}
