use super::prepare::Prepare;
use crate::postgres::protocol::{self, Execute, Message, Sync};
use std::io;

impl<'a> Prepare<'a> {
    pub async fn execute(self) -> io::Result<u64> {
        protocol::bind::trailer(
            &mut self.connection.wbuf,
            self.bind_state,
            self.bind_values,
            &[],
        );

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
