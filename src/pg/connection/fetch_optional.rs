use super::{PgRawConnection, PgRow};
use crate::pg::protocol::Message;
use std::io;

pub async fn fetch_optional<'a>(conn: &'a mut PgRawConnection) -> io::Result<Option<PgRow>> {
    conn.flush().await?;

    let mut row: Option<PgRow> = None;

    while let Some(message) = conn.receive().await? {
        match message {
            Message::BindComplete
            | Message::ParseComplete
            | Message::PortalSuspended
            | Message::CloseComplete
            | Message::CommandComplete(_) => {}

            Message::DataRow(body) => {
                row = Some(PgRow(body));
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
