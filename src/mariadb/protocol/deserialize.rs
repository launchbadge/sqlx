use super::decode::Decoder;
use crate::mariadb::connection::{ConnContext, Connection};
use bytes::Bytes;
use failure::Error;

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
