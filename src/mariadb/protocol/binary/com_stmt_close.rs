use crate::{
    io::BufMut,
    mariadb::{
        io::BufMutExt,
        protocol::{binary::BinaryProtocol, Capabilities, Encode},
    },
};
use byteorder::LittleEndian;

/// Closes a previously prepared statement.
#[derive(Debug)]
pub struct ComStmtClose {
    statement_id: i32,
}

impl Encode for ComStmtClose {
    fn encode(&self, buf: &mut Vec<u8>, _: Capabilities) {
        // COM_STMT_CLOSE : int<1>
        buf.put_u8(BinaryProtocol::ComStmtClose as u8);

        // statement_id : int<4>
        buf.put_i32::<LittleEndian>(self.statement_id);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_encodes_com_stmt_close() {
        let mut buf = Vec::new();

        ComStmtClose { statement_id: 1 }.encode(&mut buf);

        assert_eq!(&buf[..], b"\x19\x01\0\0\0");
    }
}
