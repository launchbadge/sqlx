use super::super::{client::TextProtocol, serialize::Serialize};
use crate::mariadb::connection::Connection;
use failure::Error;

pub struct ComPing();

impl Serialize for ComPing {
    fn serialize<'a, 'b>(&self, ctx: &mut crate::mariadb::connection::ConnContext, encoder: &mut crate::mariadb::protocol::encode::Encoder) -> Result<(), Error> {
        encoder.encode_int_1(TextProtocol::ComPing.into());

        Ok(())
    }
}
