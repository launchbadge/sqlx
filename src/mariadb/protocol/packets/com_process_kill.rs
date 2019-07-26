use super::super::{client::TextProtocol, serialize::Serialize};
use crate::mariadb::connection::Connection;
use failure::Error;

pub struct ComProcessKill {
    pub process_id: u32,
}

impl Serialize for ComProcessKill {
    fn serialize<'a, 'b>(&self, ctx: &mut crate::mariadb::connection::ConnContext, encoder: &mut crate::mariadb::protocol::encode::Encoder) -> Result<(), Error> {
        encoder.encode_int_1(TextProtocol::ComProcessKill.into());
        encoder.encode_int_4(self.process_id);

        Ok(())
    }
}
