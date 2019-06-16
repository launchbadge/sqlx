use bytes::Bytes;
use std::io;

pub trait Decode {
    fn decode(buf: &Bytes) -> io::Result<Self>
    where
        Self: Sized;
}
