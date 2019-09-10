use super::TextProtocol;
use crate::{
    io::BufMut,
    mariadb::protocol::{Capabilities, Encode},
};

pub struct ComQuit;

impl Encode for ComQuit {
    fn encode(&self, buf: &mut Vec<u8>, _: Capabilities) {
        buf.put_u8(TextProtocol::ComQuit as u8);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_encodes_com_quit() -> std::io::Result<()> {
        let mut buf = Vec::new();

        ComQuit.encode(&mut buf, Capabilities::empty());

        assert_eq!(&buf[..], b"\x01");

        Ok(())
    }
}
