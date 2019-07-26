use super::Encode;
use std::io;

#[derive(Debug)]
pub struct Terminate;

impl Encode for Terminate {
    fn encode(&self, buf: &mut Vec<u8>) -> io::Result<()> {
        buf.push(b'X');
        buf.extend_from_slice(&4_u32.to_be_bytes());

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::{Encode, Terminate};
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
