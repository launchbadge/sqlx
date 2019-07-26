use super::super::connection::Connection;
use failure::Error;

pub trait Serialize {
    fn serialize<'a, 'b>(&self, ctx: &mut crate::connection::ConnContext, encoder: &mut crate::protocol::encode::Encoder) -> Result<(), Error>;
}
