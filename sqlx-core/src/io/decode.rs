use bytes::Bytes;
use memchr::memchr;

use crate::error::Error;

pub trait Decode
where
    Self: Sized,
{
    fn decode(buf: Bytes) -> Result<Self, Error>;
}

impl Decode for Bytes {
    #[inline]
    fn decode(buf: Bytes) -> Result<Self, Error> {
        Ok(buf)
    }
}

impl Decode for () {
    #[inline]
    fn decode(_: Bytes) -> Result<(), Error> {
        Ok(())
    }
}
