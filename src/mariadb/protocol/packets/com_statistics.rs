use super::super::{client::TextProtocol, serialize::Serialize};
use crate::mariadb::connection::Connection;
use failure::Error;

pub struct ComStatistics();

impl Serialize for ComStatistics {
    fn serialize<'a, 'b>(&self, ctx: &mut crate::mariadb::connection::ConnContext, encoder: &mut crate::mariadb::protocol::encode::Encoder) -> Result<(), Error> {
        encoder.encode_int_1(TextProtocol::ComStatistics.into());

        Ok(())
    }
}
