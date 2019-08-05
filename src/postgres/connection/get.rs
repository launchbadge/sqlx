use super::prepare::Prepare;
use crate::postgres::protocol::{self, DataRow, Message};
use std::io;

impl<'a, 'b> Prepare<'a, 'b> {
    pub async fn get(self) -> io::Result<Option<DataRow>> {
        let conn = self.finish();

        conn.flush().await?;

        let mut row: Option<DataRow> = None;

        while let Some(message) = conn.receive().await? {
            match message {
                Message::BindComplete
                | Message::ParseComplete
                | Message::PortalSuspended
                | Message::CloseComplete => {
                    // Indicates successful completion of a phase
                }

                Message::DataRow(data_row) => {
                    // note: because we used `EXECUTE 1` this will only execute once
                    row = Some(data_row);
                }

                Message::CommandComplete(_) => {}

                Message::ReadyForQuery(_) => {
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
