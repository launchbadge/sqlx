use super::super::connection::Connection;
use failure::Error;

pub trait Serialize {
    fn serialize<'a, 'b>(&self, conn: &mut Connection) -> Result<(), Error>;
}
