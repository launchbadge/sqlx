use super::{PostgresRawConnection, PostgresRow};
use crate::postgres::protocol::Message;
use std::io;

pub async fn fetch_optional<'a>(conn: &'a mut PostgresRawConnection) -> io::Result<Option<PostgresRow>> {
    conn.flush().await?;

    let mut row: Option<PostgresRow> = None;

    while let Some(message) = conn.receive().await? {
        match message {
            Message::BindComplete
            | Message::ParseComplete
            | Message::PortalSuspended
            | Message::CloseComplete
            | Message::CommandComplete(_) => {}

            Message::DataRow(body) => {
                row = Some(PostgresRow(body));
            }

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
