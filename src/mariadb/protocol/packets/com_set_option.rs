use super::super::{client::TextProtocol, serialize::Serialize};
use crate::mariadb::connection::Connection;
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
    fn serialize<'a, 'b>(&self, ctx: &mut crate::mariadb::connection::ConnContext, encoder: &mut crate::mariadb::protocol::encode::Encoder) -> Result<(), Error> {
        encoder.encode_int_1(TextProtocol::ComSetOption.into());
        encoder.encode_int_2(self.option.into());

        Ok(())
    }
}
