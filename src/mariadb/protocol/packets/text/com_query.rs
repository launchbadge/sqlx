use crate::mariadb::{BufMutExt, Encode};
use crate::io::BufMut;

pub struct ComQuery<'a> {
    pub sql_statement: &'a str,
}

impl<'a> Encode for ComQuery<'a> {
    fn encode(&self, buf: &mut Vec<u8>) {
        buf.put_u8(super::TextProtocol::ComQuery as u8);
        buf.put_str(&self.sql_statement);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io;

    #[test]
    fn it_encodes_com_query() -> io::Result<()> {
        let mut buf = Vec::with_capacity(1024);

        ComQuery {
            sql_statement: "SELECT * FROM users",
        }
        .encode(&mut buf);

        assert_eq!(&buf[..], b"\x14\0\0\x00\x03SELECT * FROM users");

        Ok(())
    }
}
