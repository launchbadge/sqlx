use crate::mariadb::{BufMut, ConnContext, Connection, Encode};
use failure::Error;

#[derive(Debug)]
pub struct ComStmtFetch {
    pub stmt_id: i32,
    pub rows: u32,
}

impl Encode for ComStmtFetch {
    fn encode(&self, buf: &mut Vec<u8>, ctx: &mut ConnContext) -> Result<(), Error> {
        buf.alloc_packet_header();
        buf.seq_no(0);

        buf.put_int_u8(super::BinaryProtocol::ComStmtFetch as u8);
        buf.put_int_i32(self.stmt_id);
        buf.put_int_u32(self.rows);

        buf.put_length();

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_encodes_com_stmt_fetch() -> Result<(), failure::Error> {
        let mut buf = Vec::with_capacity(1024);
        let mut ctx = ConnContext::new();

        ComStmtFetch {
            stmt_id: 1,
            rows: 10,
        }
        .encode(&mut buf, &mut ctx)?;

        assert_eq!(&buf[..], b"\x09\0\0\x00\x1C\x01\0\0\0\x0A\0\0\0");

        Ok(())
    }
}
