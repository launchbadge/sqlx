use super::PgRawConnection;
use crate::pg::protocol::Message;
use std::io;

pub async fn execute(conn: &mut PgRawConnection) -> io::Result<u64> {
    conn.flush().await?;

    let mut rows = 0;

    while let Some(message) = conn.receive().await? {
        match message {
            Message::BindComplete | Message::ParseComplete | Message::DataRow(_) => {}

            Message::CommandComplete(body) => {
                rows = body.rows;
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
