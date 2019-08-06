use crate::mariadb::{BufMut, ConnContext, Connection, Encode};
use bytes::Bytes;
use failure::Error;

#[derive(Debug)]
pub struct ComStmtPrepare {
    pub statement: Bytes,
}

impl Encode for ComStmtPrepare {
    fn encode(&self, buf: &mut Vec<u8>, ctx: &mut ConnContext) -> Result<(), Error> {
        buf.alloc_packet_header();
        buf.seq_no(0);

        buf.put_int_u8(super::BinaryProtocol::ComStmtPrepare as u8);
        buf.put_string_eof(&self.statement);

        buf.put_length();

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_encodes_com_stmt_prepare() -> Result<(), failure::Error> {
        let mut buf = Vec::with_capacity(1024);
        let mut ctx = ConnContext::new();

        ComStmtPrepare {
            statement: Bytes::from_static(b"SELECT * FROM users WHERE username = ?"),
        }
        .encode(&mut buf, &mut ctx)?;

        assert_eq!(
            &buf[..],
            Bytes::from_static(b"\x27\0\0\x00\x16SELECT * FROM users WHERE username = ?")
        );

        Ok(())
    }
}
