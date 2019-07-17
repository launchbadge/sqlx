use super::decode::Decoder;
use failure::Error;
use crate::connection::Connection;
use bytes::Bytes;

pub struct DeContext<'a> {
    pub conn: &'a mut Connection,
    pub decoder: Decoder<'a>,
}

impl<'a> DeContext<'a> {
    pub fn new(conn: &'a mut Connection, buf: &'a Bytes) -> Self {
        DeContext {
            conn,
            decoder: Decoder::new(&buf),
        }
    }
}

pub trait Deserialize: Sized {
    fn deserialize(ctx: &mut DeContext) -> Result<Self, Error>;
}
