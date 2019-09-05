use crate::mariadb::{Encode};
use crate::io::BufMut;

pub struct ComSleep();

impl Encode for ComSleep {
    fn encode(&self, buf: &mut Vec<u8>) {
        buf.put_u8(super::TextProtocol::ComSleep as u8);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_encodes_com_sleep() -> std::io::Result<()> {
        let mut buf = Vec::with_capacity(1024);

        ComSleep().encode(&mut buf);

        assert_eq!(&buf[..], b"\x01\0\0\x00\x00");

        Ok(())
    }
}
