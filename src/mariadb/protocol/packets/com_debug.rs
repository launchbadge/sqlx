use crate::mariadb::{Connection, TextProtocol, Serialize};
use failure::Error;

pub struct ComDebug();

impl Serialize for ComDebug {
    fn serialize<'a, 'b>(&self, ctx: &mut crate::mariadb::connection::ConnContext, encoder: &mut crate::mariadb::protocol::encode::Encoder) -> Result<(), Error> {
        encoder.encode_int_1(TextProtocol::ComDebug.into());

        Ok(())
    }
}
