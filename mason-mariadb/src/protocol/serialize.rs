use super::types::Capabilities;
use failure::Error;
use super::encode::Encoder;

pub trait Serialize {
    fn serialize<'a, 'b>(
        &self,
        encoder: &'b mut Encoder<'a>,
        server_capabilities: &Capabilities,
    ) -> Result<(), Error>;
}
