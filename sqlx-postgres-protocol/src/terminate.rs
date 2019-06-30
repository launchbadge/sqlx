use crate::Encode;
use bytes::BufMut;
use std::io;

#[derive(Debug)]
pub struct Terminate;

impl Encode for Terminate {
    fn encode(&self, buf: &mut Vec<u8>) -> io::Result<()> {
        buf.reserve(5);
        buf.put_u8(b'X');
        buf.put_u32_be(4);

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::Terminate;
    use crate::Encode;
    use std::io;

    const TERMINATE: &[u8] = b"X\0\0\0\x04";

    #[test]
    fn it_encodes_terminate() -> io::Result<()> {
        let message = Terminate;

        let mut buf = Vec::new();
        message.encode(&mut buf)?;

        assert_eq!(&*buf, TERMINATE);

        Ok(())
    }
}
