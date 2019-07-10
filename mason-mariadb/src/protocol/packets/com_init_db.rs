use super::super::{
    client::TextProtocol, encode::Encoder, serialize::Serialize, types::Capabilities,
};
use bytes::Bytes;
use failure::Error;

pub struct ComInitDb {
    pub schema_name: Bytes,
}

impl Serialize for ComInitDb {
    fn serialize<'a, 'b>(
        &self,
        encoder: &mut Encoder,
        _server_capabilities: &Capabilities,
    ) -> Result<(), Error> {
        encoder.encode_int_1(TextProtocol::ComInitDb.into());
        encoder.encode_string_null(&self.schema_name);

        Ok(())
    }
}
