use crate::mariadb::{BufMut, ConnContext, Connection, Encode};
use bytes::Bytes;
use failure::Error;

pub struct ComQuery {
    pub sql_statement: Bytes,
}

impl Encode for ComQuery {
    fn encode(&self, buf: &mut Vec<u8>, ctx: &mut ConnContext) -> Result<(), Error> {
        buf.alloc_packet_header();
        buf.seq_no(0);

        buf.put_int_u8(super::TextProtocol::ComQuery as u8);
        buf.put_string_eof(&self.sql_statement);

        buf.put_length();

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_encodes_com_query() -> Result<(), failure::Error> {
        let mut buf = Vec::with_capacity(1024);
        let mut ctx = ConnContext::new();

        ComQuery {
            sql_statement: Bytes::from_static(b"SELECT * FROM users"),
        }
        .encode(&mut buf, &mut ctx)?;

        assert_eq!(&buf[..], b"\x14\0\0\x00\x03SELECT * FROM users");

        Ok(())
    }
}
