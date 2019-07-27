use super::super::connection::Connection;
use failure::Error;

pub trait Serialize {
    fn serialize<'a, 'b>(&self, ctx: &mut crate::mariadb::connection::ConnContext, encoder: &mut crate::mariadb::protocol::encode::Encoder) -> Result<(), Error>;
}
