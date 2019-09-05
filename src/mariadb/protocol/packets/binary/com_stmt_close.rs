use crate::mariadb::{Encode};
use byteorder::LittleEndian;
use crate::mariadb::io::BufMutExt;
use crate::io::BufMut;

#[derive(Debug)]
pub struct ComStmtClose {
    stmt_id: i32,
}

impl Encode for ComStmtClose {
    fn encode(&self, buf: &mut Vec<u8>) {
        buf.put_u8(super::BinaryProtocol::ComStmtClose as u8);
        buf.put_i32::<LittleEndian>(self.stmt_id);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_encodes_com_stmt_close() {
        let mut buf = Vec::with_capacity(1024);

        ComStmtClose { stmt_id: 1 }.encode(&mut buf);

        assert_eq!(&buf[..], b"\x05\0\0\x00\x19\x01\0\0\0");
    }
}
