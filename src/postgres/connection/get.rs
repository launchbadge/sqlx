use super::prepare::Prepare;
use crate::{
    postgres::{
        protocol::{self, DataRow, Message},
        Postgres,
    },
    row::{FromRow, Row},
    types::SqlType,
};
use std::io;

// TODO: Think through how best to handle null _rows_ and null _values_

impl<'a, 'b> Prepare<'a, 'b> {
    #[inline]
    pub async fn get<Record, T>(self) -> io::Result<T>
    where
        T: FromRow<Postgres, Record>,
    {
        Ok(T::from_row(self.get_raw().await?.unwrap()))
    }

    // TODO: Better name?
    // TODO: Should this be public?
    async fn get_raw(self) -> io::Result<Option<Row<Postgres>>> {
        let conn = self.finish();

        conn.flush().await?;

        let mut row: Option<Row<Postgres>> = None;

        while let Some(message) = conn.receive().await? {
            match message {
                Message::BindComplete
                | Message::ParseComplete
                | Message::PortalSuspended
                | Message::CloseComplete => {
                    // Indicates successful completion of a phase
                }

                Message::DataRow(body) => {
                    // note: because we used `EXECUTE 1` this will only execute once
                    row = Some(Row::<Postgres>(body));
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
