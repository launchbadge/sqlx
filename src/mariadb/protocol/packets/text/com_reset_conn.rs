use crate::mariadb::Encode;
use crate::io::BufMut;

pub struct ComResetConnection();

impl Encode for ComResetConnection {
    fn encode(&self, buf: &mut Vec<u8>) {
        buf.put_u8(super::TextProtocol::ComResetConnection as u8);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_encodes_com_reset_conn() -> std::io::Result<()> {
        let mut buf = Vec::with_capacity(1024);

        ComResetConnection().encode(&mut buf);

        assert_eq!(&buf[..], b"\x01\0\0\x00\x1F");

        Ok(())
    }
}
