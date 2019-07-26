use super::super::{client::TextProtocol, serialize::Serialize};
use crate::connection::Connection;
use failure::Error;

pub struct ComSleep();

impl Serialize for ComSleep {
    fn serialize<'a, 'b>(&self, ctx: &mut crate::connection::ConnContext, encoder: &mut crate::protocol::encode::Encoder) -> Result<(), Error> {
        encoder.encode_int_1(TextProtocol::ComSleep.into());

        Ok(())
    }
}
