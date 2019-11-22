use std::io;

pub trait Decode {
    fn decode(src: &[u8]) -> crate::Result<Self>
    where
        Self: Sized;
}
