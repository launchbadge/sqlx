use super::super::{client::TextProtocol, encode::*, serialize::Serialize, types::Capabilities};
use bytes::BytesMut;
use failure::Error;

pub struct ComQuery {
    pub sql_statement: Bytes,
}

impl Serialize for ComQuery {
    fn serialize(
        &self,
        buf: &mut BytesMut,
        _server_capabilities: &Capabilities,
    ) -> Result<(), Error> {
        encode_int_1(buf, TextProtocol::ComQuery.into());
        encode_string_eof(buf, &self.sql_statement);

        Ok(())
    }
}
