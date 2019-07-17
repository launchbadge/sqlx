use super::Connection;
use sqlx_postgres_protocol::{Bind, Execute, Message, Parse, Sync};
use std::io;

pub struct Prepare<'a> {
    connection: &'a mut Connection,
}

#[inline]
pub fn prepare<'a, 'b>(connection: &'a mut Connection, query: &'b str) -> Prepare<'a> {
    // TODO: Use a hash map to cache the parse
    // TODO: Use named statements
    connection.send(Parse::new("", query, &[]));

    Prepare { connection }
}

impl<'a> Prepare<'a> {
    #[inline]
    pub fn bind<'b>(self, value: &'b [u8]) -> Self {
        // TODO: Encode the value here onto the wbuf
        self
    }

    #[inline]
    pub async fn execute(self) -> io::Result<u64> {
        // FIXME: Break this up into BindHeader, BindValue, and BindTrailer
        self.connection.send(Bind::new("", "", &[], &[0, 0], &[]));
        self.connection.send(Execute::new("", 0));
        self.connection.send(Sync);
        self.connection.flush().await?;

        let mut rows = 0;

        while let Some(message) = self.connection.receive().await? {
            match message {
                Message::BindComplete | Message::ParseComplete => {
                    // Indicates successful completion of a phase
                }

                Message::DataRow(_) => {
                    // This is EXECUTE so we are ignoring any potential results
                }

                Message::CommandComplete(body) => {
                    rows = body.rows();
                }

                Message::ReadyForQuery(_) => {
                    // Successful completion of the whole cycle
                    return Ok(rows);
                }

                message => {
                    unimplemented!("received {:?} unimplemented message", message);
                }
            }
        }

        // FIXME: This is an end-of-file error. How we should bubble this up here?
        unreachable!()
    }
}
