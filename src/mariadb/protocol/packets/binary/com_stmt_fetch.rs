use crate::mariadb::{Encode};
use crate::io::BufMut;
use byteorder::LittleEndian;

#[derive(Debug)]
pub struct ComStmtFetch {
    pub stmt_id: i32,
    pub rows: u32,
}

impl Encode for ComStmtFetch {
    fn encode(&self, buf: &mut Vec<u8>) {
        buf.put_u8(super::BinaryProtocol::ComStmtFetch as u8);
        buf.put_i32::<LittleEndian>(self.stmt_id);
        buf.put_u32::<LittleEndian>(self.rows);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_encodes_com_stmt_fetch(){
        let mut buf = Vec::with_capacity(1024);

        ComStmtFetch {
            stmt_id: 1,
            rows: 10,
        }
        .encode(&mut buf);

        assert_eq!(&buf[..], b"\x09\0\0\x00\x1C\x01\0\0\0\x0A\0\0\0");
    }
}
