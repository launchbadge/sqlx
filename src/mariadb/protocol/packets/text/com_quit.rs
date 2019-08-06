use crate::mariadb::{BufMut, ConnContext, Connection, Encode};
use failure::Error;

pub struct ComQuit();

impl Encode for ComQuit {
    fn encode(&self, buf: &mut Vec<u8>, ctx: &mut ConnContext) -> Result<(), Error> {
        buf.alloc_packet_header();
        buf.seq_no(0);

        buf.put_int_u8(super::TextProtocol::ComQuit as u8);

        buf.put_length();

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_encodes_com_quit() -> Result<(), failure::Error> {
        let mut buf = Vec::with_capacity(1024);
        let mut ctx = ConnContext::new();

        ComQuit().encode(&mut buf, &mut ctx)?;

        assert_eq!(&buf[..], b"\x01\0\0\x00\x01");

        Ok(())
    }
}
