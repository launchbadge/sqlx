use super::{PgRawConnection, PgRow};
use crate::pg::protocol::Message;
use futures_core::stream::Stream;
use std::io;

pub fn fetch<'a>(
    conn: &'a mut PgRawConnection,
) -> impl Stream<Item = Result<PgRow, io::Error>> + 'a {
    async_stream::try_stream! {
        conn.flush().await?;

        while let Some(message) = conn.receive().await? {
            match message {
                Message::BindComplete
                | Message::ParseComplete
                | Message::PortalSuspended
                | Message::CloseComplete
                | Message::CommandComplete(_) => {}

                Message::DataRow(body) => {
                    yield PgRow(body);
                }

                Message::ReadyForQuery(_) => {
                    return;
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
