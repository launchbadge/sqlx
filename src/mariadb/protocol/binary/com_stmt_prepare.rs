use crate::{
    io::BufMut,
    mariadb::{BufMutExt, Encode},
};

#[derive(Debug)]
pub struct ComStmtPrepare<'a> {
    pub statement: &'a str,
}

impl<'a> Encode for ComStmtPrepare<'a> {
    fn encode(&self, buf: &mut Vec<u8>) {
        buf.put_u8(super::BinaryProtocol::ComStmtPrepare as u8);
        buf.put_str(&self.statement);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_encodes_com_stmt_prepare() -> std::io::Result<()> {
        let mut buf = Vec::with_capacity(1024);

        ComStmtPrepare {
            statement: "SELECT * FROM users WHERE username = ?",
        }
        .encode(&mut buf);

        assert_eq!(
            &buf[..],
            "\x27\0\0\x00\x16SELECT * FROM users WHERE username = ?"
        );

        Ok(())
    }
}
