use super::super::{client::TextProtocol, encode::*, serialize::Serialize, types::Capabilities};
use bytes::BytesMut;
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
    fn serialize(
        &self,
        buf: &mut BytesMut,
        _server_capabilities: &Capabilities,
    ) -> Result<(), Error> {
        encode_int_1(buf, TextProtocol::ComSetOption.into());
        encode_int_2(buf, self.option.into());

        Ok(())
    }
}
