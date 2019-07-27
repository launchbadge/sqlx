use super::super::{client::TextProtocol, serialize::Serialize};
use crate::mariadb::connection::Connection;
use bytes::Bytes;
use failure::Error;

pub struct ComQuery {
    pub sql_statement: Bytes,
}

impl Serialize for ComQuery {
    fn serialize<'a, 'b>(&self, ctx: &mut crate::mariadb::connection::ConnContext, encoder: &mut crate::mariadb::protocol::encode::Encoder) -> Result<(), Error> {
        encoder.encode_int_1(TextProtocol::ComQuery.into());
        encoder.encode_string_eof(&self.sql_statement);

        Ok(())
    }
}
