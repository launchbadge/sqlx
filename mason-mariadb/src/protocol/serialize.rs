use super::{super::connection::Connection, encode::Encoder, types::Capabilities};
use failure::Error;

pub trait Serialize {
    fn serialize<'a, 'b>(
        &self,
        encoder: &'b mut Encoder,
        server_capabilities: &Capabilities,
    ) -> Result<(), Error>;
}

pub struct Serializer<'a> {
    pub conn: &'a mut Connection,
}

impl<'a> Serializer<'a> {
    #[inline]
    pub fn new(conn: &'a mut Connection) -> Self {
        Serializer {
            conn,
        }
    }

    #[inline]
    pub fn serialize<S: Serialize>(&mut self, message: S) -> Result<(), Error> {
        message.serialize(&mut self.conn.encoder, &self.conn.capabilities)
    }
}
