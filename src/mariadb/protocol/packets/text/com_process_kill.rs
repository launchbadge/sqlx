use crate::mariadb::{BufMut, ConnContext, Connection, Encode};
use failure::Error;

pub struct ComProcessKill {
    pub process_id: u32,
}

impl Encode for ComProcessKill {
    fn encode(&self, buf: &mut Vec<u8>, ctx: &mut ConnContext) -> Result<(), Error> {
        buf.alloc_packet_header();
        buf.seq_no(0);

        buf.put_int_u8(super::TextProtocol::ComProcessKill.into());
        buf.put_int_u32(self.process_id);

        buf.put_length();

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_encodes_com_process_kill() -> Result<(), failure::Error> {
        let mut buf = Vec::with_capacity(1024);
        let mut ctx = ConnContext::new();

        ComProcessKill { process_id: 1 }.encode(&mut buf, &mut ctx)?;

        assert_eq!(&buf[..], b"\x05\0\0\x00\x0C\x01\0\0\0");

        Ok(())
    }
}
