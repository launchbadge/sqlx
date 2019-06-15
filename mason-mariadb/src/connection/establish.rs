use super::Connection;
use crate::protocol::{
    server::Message as ServerMessage,
};
use futures::StreamExt;
use mason_core::ConnectOptions;
use std::io;

pub async fn establish<'a, 'b: 'a>(
    conn: &'a mut Connection,
    options: ConnectOptions<'b>,
) -> io::Result<()> {
    // The actual connection establishing

    // if let Some(message) = conn.incoming.next().await {
    //     if let Some(ServerMessage::InitialHandshakePacket(message)) = message {

    //     } else {
    //         unimplemented!("received {:?} unimplemented message", message);
    //     }
    // }

    Ok(())
}
