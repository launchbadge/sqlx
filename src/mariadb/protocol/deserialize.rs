use crate::mariadb::{Framed, Decoder, ConnContext, Connection, ColumnDefPacket};
use bytes::Bytes;
use failure::Error;

// A wrapper around a connection context to prevent
// deserializers from touching the stream, yet still have
// access to the connection context.
// Mainly used to simply to simplify number of parameters for deserializing functions
pub struct DeContext<'a> {
    pub ctx: &'a mut ConnContext,
    pub stream: Option<&'a mut Framed>,
    pub decoder: Decoder,
    pub columns: Option<u64>,
    pub column_defs: Option<Vec<ColumnDefPacket>>,
}

impl<'a> DeContext<'a> {
    pub fn new(conn: &'a mut ConnContext, buf: Bytes) -> Self {
        DeContext { ctx: conn, stream: None, decoder: Decoder::new(buf), columns: None , column_defs: None }
    }

    pub fn with_stream(conn: &'a mut ConnContext, stream: &'a mut Framed) -> Self {
        DeContext {
            ctx: conn,
            stream: Some(stream),
            decoder: Decoder::new(Bytes::new()),
            columns: None ,
            column_defs: None
        }
    }

    pub async fn next_packet(&mut self) -> Result<(), failure::Error> {
        if let Some(stream) = &mut self.stream {
            println!("Called next packet");
            self.decoder = Decoder::new(stream.next_packet().await?);

            Ok(())
        } else {
            failure::bail!("Calling next_packet on DeContext with no stream provided")
        }
    }
}

pub trait Deserialize: Sized {
    fn deserialize(ctx: &mut DeContext) -> Result<Self, Error>;
}
