use crate::mariadb::{Encode};
use crate::io::BufMut;

pub struct ComStatistics();

impl Encode for ComStatistics {
    fn encode(&self, buf: &mut Vec<u8>) {
        buf.put_u8(super::TextProtocol::ComStatistics.into());
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_encodes_com_statistics() -> std::io::Result<()> {
        let mut buf = Vec::with_capacity(1024);

        ComStatistics().encode(&mut buf);

        assert_eq!(&buf[..], b"\x01\0\0\x00\x09");

        Ok(())
    }
}
