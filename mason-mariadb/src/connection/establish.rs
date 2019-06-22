use super::Connection;
use crate::protocol::{
    server::Message as ServerMessage,
    server::InitialHandshakePacket,
    server::Deserialize
};
use futures::StreamExt;
use mason_core::ConnectOptions;
use std::io;
use failure::Error;

pub async fn establish<'a, 'b: 'a>(
    conn: &'a mut Connection,
    options: ConnectOptions<'b>,
) -> Result<(), Error> {
    // The actual connection establishing
//
     if let Some(message) = conn.incoming.next().await {
//         return
//         match message {
//             ServerMessage::InitialHandshakePacket(message) => {
//
//             },
//             _ => unimplemented!("received {:?} unimplemented message", message),
//         }
         Ok(())
     } else {
         Err(failure::err_msg("Failed to connect"))
     }
}

#[cfg(test)]
mod test {
    use super::*;
    use failure::Error;

    #[runtime::test]
    async fn it_connects() -> Result<(), Error> {
        Connection::establish(ConnectOptions {
            host: "localhost",
            port: 3306,
            user: Some("root"),
            database: None,
            password: None,
        }).await?;

        Ok(())
    }
}

