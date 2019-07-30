use crate::mariadb::{TextProtocol, Serialize, Connection};
use bytes::Bytes;
use failure::Error;

pub struct ComQuery {
    pub sql_statement: Bytes,
}

impl Serialize for ComQuery {
    fn serialize<'a, 'b>(&self, ctx: &mut crate::mariadb::connection::ConnContext, encoder: &mut crate::mariadb::protocol::encode::Encoder) -> Result<(), Error> {
        encoder.alloc_packet_header();
        encoder.seq_no(0);

        encoder.encode_int_u8(TextProtocol::ComQuery.into());
        encoder.encode_string_eof(&self.sql_statement);

        encoder.encode_length();

        Ok(())
    }
}
