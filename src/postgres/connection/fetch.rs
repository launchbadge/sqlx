use super::{PostgresRawConnection, PostgresRow};
use crate::{error::Error, postgres::protocol::Message};
use futures_core::stream::Stream;
use std::io;

pub fn fetch<'a>(
    conn: &'a mut PostgresRawConnection,
) -> impl Stream<Item = Result<PostgresRow, Error>> + 'a {
    async_stream::try_stream! {
        conn.stream.flush().await.map_err(Error::from)?;

        while let Some(message) = conn.receive().await? {
            match message {
                Message::BindComplete
                | Message::ParseComplete
                | Message::PortalSuspended
                | Message::CloseComplete
                | Message::CommandComplete(_) => {}

                Message::DataRow(body) => {
                    yield PostgresRow(body);
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
