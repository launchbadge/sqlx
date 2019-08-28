use super::PostgresRawConnection;
use crate::{error::Error, postgres::protocol::Message};
use std::io;

pub async fn execute(conn: &mut PostgresRawConnection) -> Result<u64, Error> {
    conn.stream.flush().await?;

    let mut rows = 0;

    while let Some(message) = conn.receive().await? {
        match message {
            Message::BindComplete | Message::ParseComplete | Message::DataRow(_) => {}

            Message::CommandComplete(body) => {
                rows = body.affected_rows();
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
