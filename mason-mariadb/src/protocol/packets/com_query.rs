use super::super::{
    client::TextProtocol, encode::Encoder, serialize::Serialize, types::Capabilities,
};
use bytes::Bytes;
use failure::Error;

pub struct ComQuery {
    pub sql_statement: Bytes,
}

impl Serialize for ComQuery {
    fn serialize<'a, 'b>(
        &self,
        encoder: &mut Encoder,
        _server_capabilities: &Capabilities,
    ) -> Result<(), Error> {
        encoder.encode_int_1(TextProtocol::ComQuery.into());
        encoder.encode_string_eof(&self.sql_statement);

        Ok(())
    }
}
