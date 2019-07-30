use super::decode::Decoder;
use crate::mariadb::connection::{ConnContext, Connection};
use bytes::Bytes;
use failure::Error;

// A wrapper around a connection context to prevent
// deserializers from touching the stream, yet still have
// access to the connection context.
// Mainly used to simply to simplify number of parameters for deserializing functions
pub struct DeContext<'a> {
    pub conn: &'a mut ConnContext,
    pub decoder: Decoder<'a>,
    pub columns: Option<usize>,
}

impl<'a> DeContext<'a> {
    pub fn new(conn: &'a mut ConnContext, buf: &'a Bytes) -> Self {
        DeContext { conn, decoder: Decoder::new(&buf), columns: None }
    }
}

pub trait Deserialize: Sized {
    fn deserialize(ctx: &mut DeContext) -> Result<Self, Error>;
}
