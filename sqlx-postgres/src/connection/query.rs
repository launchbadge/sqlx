use super::Connection;
use crate::protocol::{client::Query, server::Message as ServerMessage};
use futures::StreamExt;
use std::io;

pub async fn query<'a, 'b: 'a>(conn: &'a mut Connection, query: &'a str) -> io::Result<()> {
    conn.send(Query { query }).await?;

    // FIXME: This feels like it could be reduced (see other connection flows)
    while let Some(message) = conn.incoming.next().await {
        match message {
            ServerMessage::ReadyForQuery(_) => {
                break;
            }

            ServerMessage::CommandComplete(body) => {
                log::debug!("command complete: {}", body.tag()?);
            }

            _ => {
                unimplemented!("received {:?} unimplemented message", message);
            }
        }
    }

    Ok(())
}
