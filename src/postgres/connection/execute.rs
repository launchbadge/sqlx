use super::prepare::Prepare;
use crate::postgres::protocol::{self, Message};
use std::io;

impl<'a> Prepare<'a> {
    pub async fn execute(self) -> io::Result<u64> {
        // protocol::bind::trailer(
        //     &mut self.connection.wbuf,
        //     self.bind_state,
        //     self.bind_values,
        //     &[],
        // );

        // protocol::execute(&mut self.connection.wbuf, "", 0);
        // protocol::sync(&mut self.connection.wbuf);

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
