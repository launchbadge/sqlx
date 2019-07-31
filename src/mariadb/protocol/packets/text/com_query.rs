use crate::mariadb::{Connection, Serialize};
use bytes::Bytes;
use failure::Error;

pub struct ComQuery {
    pub sql_statement: Bytes,
}

impl Serialize for ComQuery {
    fn serialize<'a, 'b>(&self, ctx: &mut crate::mariadb::ConnContext, encoder: &mut crate::mariadb::Encoder) -> Result<(), Error> {
        encoder.alloc_packet_header();
        encoder.seq_no(0);

        encoder.encode_int_u8(super::TextProtocol::ComQuery.into());
        encoder.encode_string_eof(&self.sql_statement);

        encoder.encode_length();

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mariadb::{ConnContext, Encoder};

    #[test]
    fn it_encodes_com_query() -> Result<(), failure::Error> {
        let mut encoder = Encoder::new(128);
        let mut ctx = ConnContext::new();

        ComQuery {
            sql_statement: Bytes::from_static(b"SELECT * FROM users"),
        }.serialize(&mut ctx, &mut encoder)?;

        assert_eq!(&encoder.buf[..], b"\x14\0\0\x00\x03SELECT * FROM users");

        Ok(())
    }
}

