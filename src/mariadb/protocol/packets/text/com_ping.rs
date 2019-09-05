use crate::mariadb::{Encode};
use crate::io::BufMut;

pub struct ComPing();

impl Encode for ComPing {
    fn encode(&self, buf: &mut Vec<u8>) {
        buf.put_u8(super::TextProtocol::ComPing.into());
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io;

    #[test]
    fn it_encodes_com_ping() -> io::Result<()> {
        let mut buf = Vec::with_capacity(1024);

        ComPing().encode(&mut buf);

        assert_eq!(&buf[..], b"\x01\0\0\x00\x0E");

        Ok(())
    }
}
