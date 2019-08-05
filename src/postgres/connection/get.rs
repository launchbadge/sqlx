use super::prepare::Prepare;
use crate::{
    postgres::protocol::{self, DataRow, Message},
    row::Row,
};
use std::io;

impl<'a, 'b> Prepare<'a, 'b> {
    pub async fn get(self) -> io::Result<Option<Row>> {
        let conn = self.finish();

        conn.flush().await?;

        let mut raw: Option<DataRow> = None;

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
                    raw = Some(data_row);
                }

                Message::CommandComplete(_) => {}

                Message::ReadyForQuery(_) => {
                    return Ok(raw.map(Row));
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
