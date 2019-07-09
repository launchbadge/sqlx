use super::super::{client::TextProtocol, encode::*, serialize::Serialize, types::Capabilities};
use bytes::BytesMut;
use failure::Error;

pub struct ComDebug();

impl Serialize for ComDebug {
    fn serialize(
        &self,
        buf: &mut BytesMut,
        _server_capabilities: &Capabilities,
    ) -> Result<(), Error> {
        encode_int_1(buf, TextProtocol::ComDebug.into());

        Ok(())
    }
}
