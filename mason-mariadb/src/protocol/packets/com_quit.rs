use super::super::{client::TextProtocol, serialize::Serialize};
use crate::connection::Connection;
use failure::Error;

pub struct ComQuit();

impl Serialize for ComQuit {
    fn serialize<'a, 'b>(&self, conn: &mut Connection) -> Result<(), Error> {
        conn.encoder.encode_int_1(TextProtocol::ComQuit.into());

        Ok(())
    }
}
