use super::prepare::Prepare;
use crate::postgres::protocol::{self, Message};
use std::io;

impl<'a, 'b> Prepare<'a, 'b> {
    pub async fn execute(self) -> io::Result<u64> {
        let conn = self.finish();

        conn.flush().await?;

        let mut rows = 0;

        while let Some(message) = conn.receive().await? {
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
