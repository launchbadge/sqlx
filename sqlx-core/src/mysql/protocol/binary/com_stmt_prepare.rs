use crate::{
    io::BufMut,
    mysql::{
        io::BufMutExt,
        protocol::{Capabilities, Encode},
    },
};

#[derive(Debug)]
pub struct ComStmtPrepare<'a> {
    pub statement: &'a str,
}

impl Encode for ComStmtPrepare<'_> {
    fn encode(&self, buf: &mut Vec<u8>, _: Capabilities) {
        // COM_STMT_PREPARE : int<1>
        buf.put_u8(super::BinaryProtocol::ComStmtPrepare as u8);

        // SQL Statement : string<EOF>
        buf.put_str(&self.statement);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_encodes_com_stmt_prepare() {
        let mut buf = Vec::new();

        ComStmtPrepare {
            statement: "SELECT * FROM users WHERE username = ?",
        }
        .encode(&mut buf, Capabilities::empty());

        assert_eq!(&buf[..], &b"\x16SELECT * FROM users WHERE username = ?"[..]);
    }
}
