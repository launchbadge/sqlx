use super::super::{client::TextProtocol, encode::*, serialize::Serialize, types::Capabilities};
use bytes::BytesMut;
use failure::Error;

pub struct ComInitDb {
    pub schema_name: Bytes,
}

impl Serialize for ComInitDb {
    fn serialize(
        &self,
        buf: &mut BytesMut,
        _server_capabilities: &Capabilities,
    ) -> Result<(), Error> {
        encode_int_1(buf, TextProtocol::ComInitDb.into());
        encode_string_null(buf, &self.schema_name);

        Ok(())
    }
}
