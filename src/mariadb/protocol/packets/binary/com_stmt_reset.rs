use crate::mariadb::{Encode};
use crate::io::BufMut;
use byteorder::LittleEndian;

#[derive(Debug)]
pub struct ComStmtReset {
    pub stmt_id: i32,
}

impl crate::mariadb::Encode for ComStmtReset {
    fn encode(&self, buf: &mut Vec<u8>) {
        buf.put_u8(super::BinaryProtocol::ComStmtReset as u8);
        buf.put_i32::<LittleEndian>(self.stmt_id);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io;

    #[test]
    fn it_encodes_com_stmt_reset() -> io::Result<()> {
        let mut buf = Vec::with_capacity(1024);

        ComStmtReset { stmt_id: 1 }.encode(&mut buf);

        assert_eq!(&buf[..], b"\x05\0\0\x00\x1A\x01\0\0\0");

        Ok(())
    }
}
