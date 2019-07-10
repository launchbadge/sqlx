use super::decode::Decoder;
use failure::Error;
use crate::connection::Connection;

pub trait Deserialize: Sized {
    fn deserialize(conn: &mut Connection, decoder: &mut Decoder) -> Result<Self, Error>;
}
