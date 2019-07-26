use super::super::{client::TextProtocol, serialize::Serialize};
use crate::mariadb::connection::Connection;
use failure::Error;

pub struct ComResetConnection();

impl Serialize for ComResetConnection {
    fn serialize<'a, 'b>(&self, ctx: &mut crate::mariadb::connection::ConnContext, encoder: &mut crate::mariadb::protocol::encode::Encoder) -> Result<(), Error> {
        encoder.encode_int_1(TextProtocol::ComResetConnection.into());

        Ok(())
    }
}
