use crate::mariadb::{Decoder, ConnContext, Connection, ColumnDefPacket};
use bytes::Bytes;
use failure::Error;

// A wrapper around a connection context to prevent
// deserializers from touching the stream, yet still have
// access to the connection context.
// Mainly used to simply to simplify number of parameters for deserializing functions
pub struct DeContext<'a> {
    pub ctx: &'a mut ConnContext,
    pub decoder: Decoder<'a>,
    pub columns: Option<u64>,
    pub column_defs: Option<Vec<ColumnDefPacket>>,
}

impl<'a> DeContext<'a> {
    pub fn new(conn: &'a mut ConnContext, buf: &'a Bytes) -> Self {
        DeContext { ctx: conn, decoder: Decoder::new(&buf), columns: None , column_defs: None }
    }
}

pub trait Deserialize: Sized {
    fn deserialize(ctx: &mut DeContext) -> Result<Self, Error>;
}
