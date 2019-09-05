use crate::{io::BufMut, mariadb::Encode};

pub struct ComQuit();

impl Encode for ComQuit {
    fn encode(&self, buf: &mut Vec<u8>) {
        buf.put_u8(super::TextProtocol::ComQuit as u8);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_encodes_com_quit() -> std::io::Result<()> {
        let mut buf = Vec::with_capacity(1024);

        ComQuit().encode(&mut buf);

        assert_eq!(&buf[..], b"\x01\0\0\x00\x01");

        Ok(())
    }
}
