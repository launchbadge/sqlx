pub trait Decode {
    fn decode(buf: &[u8]) -> crate::Result<Self>
    where
        Self: Sized;
}
