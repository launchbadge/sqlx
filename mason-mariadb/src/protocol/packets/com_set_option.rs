use super::super::{
    client::TextProtocol, encode::Encoder, serialize::Serialize, types::Capabilities,
};
use failure::Error;

#[derive(Clone, Copy)]
pub enum SetOptionOptions {
    MySqlOptionMultiStatementsOn = 0x00,
    MySqlOptionMultiStatementsOff = 0x01,
}

pub struct ComSetOption {
    pub option: SetOptionOptions,
}

impl Serialize for ComSetOption {
    fn serialize<'a, 'b>(
        &self,
        encoder: &mut Encoder,
        _server_capabilities: &Capabilities,
    ) -> Result<(), Error> {
        encoder.encode_int_1(TextProtocol::ComSetOption.into());
        encoder.encode_int_2(self.option.into());

        Ok(())
    }
}
