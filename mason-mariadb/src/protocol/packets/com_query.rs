use super::super::{client::TextProtocol, serialize::Serialize};
use crate::connection::Connection;
use bytes::Bytes;
use failure::Error;

pub struct ComQuery {
    pub sql_statement: Bytes,
}

impl Serialize for ComQuery {
    fn serialize<'a, 'b>(&self, conn: &mut Connection) -> Result<(), Error> {
        conn.encoder.encode_int_1(TextProtocol::ComQuery.into());
        conn.encoder.encode_string_eof(&self.sql_statement);

        Ok(())
    }
}
