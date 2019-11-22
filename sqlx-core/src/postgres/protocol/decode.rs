use std::io;

pub trait Decode {
    fn decode(src: &[u8]) -> io::Result<Self>
    where
        Self: Sized;
}
