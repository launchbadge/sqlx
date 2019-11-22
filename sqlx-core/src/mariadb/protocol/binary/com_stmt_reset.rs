use crate::{
    io::BufMut,
    mariadb::protocol::{binary::BinaryProtocol, Capabilities, Encode},
};
use byteorder::LittleEndian;

#[derive(Debug)]
pub struct ComStmtReset {
    pub statement_id: u32,
}

impl Encode for ComStmtReset {
    fn encode(&self, buf: &mut Vec<u8>, _: Capabilities) {
        // COM_STMT_RESET : int<1>
        buf.put_u8(BinaryProtocol::ComStmtReset as u8);

        // statement_id : int<4>
        buf.put_u32::<LittleEndian>(self.statement_id);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_encodes_com_stmt_reset() {
        let mut buf = Vec::new();

        ComStmtReset { statement_id: 1 }.encode(&mut buf, Capabilities::empty());

        assert_eq!(&buf[..], b"\x1A\x01\0\0\0");
    }
}
