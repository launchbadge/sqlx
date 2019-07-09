use super::decode::Decoder;
use bytes::Bytes;
use failure::Error;

pub trait Deserialize: Sized {
    fn deserialize<'a, 'b>(
        buf: &'a Bytes,
        decoder: Option<&'b mut Decoder<'a>>,
    ) -> Result<Self, Error>;
}
