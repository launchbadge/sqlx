use super::Connection;
use crate::protocol::{
    server::Message as ServerMessage,
    server::InitialHandshakePacket,
    server::Deserialize,
    server::Capabilities,
    client::HandshakeResponsePacket,
    client::Serialize
};
use futures::StreamExt;
use mason_core::ConnectOptions;
use std::io;
use failure::Error;
use bytes::Bytes;

pub async fn establish<'a, 'b: 'a>(
    conn: &'a mut Connection,
    options: ConnectOptions<'b>,
) -> Result<(), Error> {
     let init_packet =  if let Some(message) = conn.incoming.next().await {
         match message {
             ServerMessage::InitialHandshakePacket(message) => {
                    Ok(message)
             },
             _ => Err(failure::err_msg("Incorrect First Packet")),
         }
     } else {
         Err(failure::err_msg("Failed to connect"))
     }?;

//     println!("{:?}", init_packet);

    let handshake = HandshakeResponsePacket {
        server_capabilities: init_packet.capabilities,
        sequence_number: 0,
        capabilities: Capabilities::from_bits_truncate(0),
        max_packet_size: 1024,
        collation: 0,
        extended_capabilities: Some(Capabilities::from_bits_truncate(0)),
        username: Bytes::from("username"),
        auth_data: None,
        auth_response_len: None,
        auth_response: None,
        database: None,
        auth_plugin_name: None,
        conn_attr_len: None,
        conn_attr: None,
    };
    conn.send(handshake).await?;

    if let Some(message) = conn.incoming.next().await {
        Ok(())
    } else {
        Err(failure::err_msg("Handshake Failed"))
    }

//    Ok(())
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

