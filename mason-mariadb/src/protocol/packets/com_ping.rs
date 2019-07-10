use super::super::{client::TextProtocol, serialize::Serialize};
use crate::connection::Connection;
use failure::Error;

pub struct ComPing();

impl Serialize for ComPing {
    fn serialize<'a, 'b>(&self, conn: &mut Connection) -> Result<(), Error> {
        conn.encoder.encode_int_1(TextProtocol::ComPing.into());

        Ok(())
    }
}
