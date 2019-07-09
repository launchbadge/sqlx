use super::types::Capabilities;
use bytes::BytesMut;
use failure::Error;

pub trait Serialize {
    fn serialize(
        &self,
        buf: &mut BytesMut,
        server_capabilities: &Capabilities,
    ) -> Result<(), Error>;
}
