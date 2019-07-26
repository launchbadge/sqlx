use super::prepare::Prepare;
use crate::postgres::protocol::{self, DataRow, Message};
use std::io;

impl<'a> Prepare<'a> {
    pub async fn get(self) -> io::Result<Option<DataRow>> {
        protocol::bind::trailer(
            &mut self.connection.wbuf,
            self.bind_state,
            self.bind_values,
            &[],
        );

        protocol::execute(&mut self.connection.wbuf, "", 0);
        protocol::sync(&mut self.connection.wbuf);

        self.connection.flush().await?;

        let mut row: Option<DataRow> = None;

        while let Some(message) = self.connection.receive().await? {
            match message {
                Message::BindComplete | Message::ParseComplete => {
                    // Indicates successful completion of a phase
                }

                Message::DataRow(data_row) => {
                    // we only care about the first result.
                    if row.is_none() {
                        row = Some(data_row);
                    }
                }

                Message::CommandComplete(_) => {}

                Message::ReadyForQuery(_) => {
                    // Successful completion of the whole cycle
                    return Ok(row);
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
