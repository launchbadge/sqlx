use std::io;

pub trait Encode {
    fn size_hint() -> usize;
    fn encode(&self, buf: &mut Vec<u8>) -> io::Result<()>;

    #[inline]
    fn to_bytes(&self) -> io::Result<Vec<u8>> {
        let mut buf = Vec::with_capacity(Self::size_hint());
        self.encode(&mut buf)?;
        Ok(buf)
    }
}
