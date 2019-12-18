use crate::{
    io::BufMut,
    mysql::protocol::{binary::BinaryProtocol, Capabilities, Encode},
};
use byteorder::LittleEndian;

// https://mariadb.com/kb/en/library/com_stmt_fetch/
/// Fetch rows from a prepared statement.
#[derive(Debug)]
pub struct ComStmtFetch {
    pub statement_id: u32,
    pub rows: u32,
}

impl Encode for ComStmtFetch {
    fn encode(&self, buf: &mut Vec<u8>, _: Capabilities) {
        // COM_STMT_FETCH : int<1>
        buf.put_u8(BinaryProtocol::ComStmtFetch as u8);

        // statement id : int<4>
        buf.put_u32::<LittleEndian>(self.statement_id);

        // number of rows to fetch : int<4>
        buf.put_u32::<LittleEndian>(self.rows);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_encodes_com_stmt_fetch() {
        let mut buf = Vec::new();

        ComStmtFetch {
            statement_id: 1,
            rows: 10,
        }
        .encode(&mut buf, Capabilities::empty());

        assert_eq!(&buf[..], b"\x1C\x01\0\0\0\x0A\0\0\0");
    }
}
