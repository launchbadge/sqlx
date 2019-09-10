use super::TextProtocol;
use crate::{
    io::BufMut,
    mariadb::protocol::{Capabilities, Encode},
};

#[derive(Debug)]
pub struct ComPing;

impl Encode for ComPing {
    fn encode(&self, buf: &mut Vec<u8>, _: Capabilities) {
        // COM_PING Header : int<1>
        buf.put_u8(TextProtocol::ComPing as u8);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_encodes_com_ping() {
        let mut buf = Vec::new();
        ComPing.encode(&mut buf, Capabilities::empty());

        assert_eq!(&buf[..], b"\x0E");
    }
}
