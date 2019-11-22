use super::TextProtocol;
use crate::{
    io::BufMut,
    mariadb::protocol::{Capabilities, Encode},
};

#[derive(Debug)]
pub struct ComDebug;

impl Encode for ComDebug {
    fn encode(&self, buf: &mut Vec<u8>, _: Capabilities) {
        // COM_DEBUG Header (0xOD) : int<1>
        buf.put_u8(TextProtocol::ComDebug as u8);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_encodes_com_debug() {
        let mut buf = Vec::new();
        ComDebug.encode(&mut buf, Capabilities::empty());

        assert_eq!(&buf[..], b"\x0D");
    }
}
