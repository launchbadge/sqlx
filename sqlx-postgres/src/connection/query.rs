use super::Connection;
use futures::StreamExt;
use sqlx_postgres_protocol::{Message, Query};
use std::io;

pub async fn query<'a: 'b, 'b>(conn: &'a mut Connection, query: &'b str) -> io::Result<()> {
    conn.send(Query::new(query)).await?;

    while let Some(message) = conn.stream.next().await {
        match message? {
            Message::ReadyForQuery(_) => {
                break;
            }

            message => {
                unimplemented!("received {:?} unimplemented message", message);
            }
        }
    }

    Ok(())
}
