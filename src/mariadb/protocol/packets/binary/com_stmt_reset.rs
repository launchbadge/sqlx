use crate::mariadb::{BufMut, ConnContext, Connection, Encode};
use failure::Error;

#[derive(Debug)]
pub struct ComStmtReset {
    pub stmt_id: i32,
}

impl crate::mariadb::Encode for ComStmtReset {
    fn encode(&self, buf: &mut Vec<u8>, ctx: &mut ConnContext) -> Result<(), Error> {
        buf.alloc_packet_header();
        buf.seq_no(0);

        buf.put_int_u8(super::BinaryProtocol::ComStmtReset as u8);
        buf.put_int_i32(self.stmt_id);

        buf.put_length();

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_encodes_com_stmt_reset() -> Result<(), failure::Error> {
        let mut buf = Vec::with_capacity(1024);
        let mut ctx = ConnContext::new();

        ComStmtReset { stmt_id: 1 }.encode(&mut buf, &mut ctx)?;

        assert_eq!(&buf[..], b"\x05\0\0\x00\x1A\x01\0\0\0");

        Ok(())
    }
}
