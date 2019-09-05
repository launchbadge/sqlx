use crate::{io::BufMut, mariadb::Encode};

pub struct ComDebug();

impl Encode for ComDebug {
    fn encode(&self, buf: &mut Vec<u8>) {
        buf.put_u8(super::TextProtocol::ComDebug as u8);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io;

    #[test]
    fn it_encodes_com_debug() -> io::Result<()> {
        let mut buf = Vec::with_capacity(1024);

        ComDebug().encode(&mut buf);

        assert_eq!(&buf[..], b"\x01\0\0\x00\x0D");

        Ok(())
    }
}
