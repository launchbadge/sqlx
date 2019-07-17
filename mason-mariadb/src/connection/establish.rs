use super::{Connection};
use crate::protocol::{
    deserialize::{Deserialize, DeContext},
    packets::{handshake_response::HandshakeResponsePacket, initial::InitialHandshakePacket},
    server::Message as ServerMessage,
    types::Capabilities,
};
use bytes::Bytes;
use failure::{err_msg, Error};
use mason_core::ConnectOptions;

pub async fn establish<'a, 'b: 'a>(
    conn: &'a mut Connection,
    options: ConnectOptions<'b>,
) -> Result<(), Error> {
    let buf = &conn.stream.next_bytes().await?;
    let mut de_ctx = DeContext::new(conn, &buf);
    let _ = InitialHandshakePacket::deserialize(&mut de_ctx)?;

    let handshake: HandshakeResponsePacket = HandshakeResponsePacket {
        // Minimum client capabilities required to establish connection
        capabilities: Capabilities::CLIENT_PROTOCOL_41,
        max_packet_size: 1024,
        extended_capabilities: Some(Capabilities::from_bits_truncate(0)),
        username: Bytes::from(options.user.unwrap_or("")),
        ..Default::default()
    };

    conn.send(handshake).await?;

    match conn.next().await? {
        Some(ServerMessage::OkPacket(message)) => {
            conn.seq_no = message.seq_no;
            Ok(())
        }

        Some(ServerMessage::ErrPacket(message)) => Err(err_msg(format!("{:?}", message))),

        Some(message) => {
            panic!("Did not receive OkPacket nor ErrPacket. Received: {:?}", message);
        }

        None => {
            panic!("Did not recieve packet");
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use failure::Error;

    #[runtime::test]
    async fn it_connects() -> Result<(), Error> {
        let mut conn = Connection::establish(ConnectOptions {
            host: "127.0.0.1",
            port: 3306,
            user: Some("root"),
            database: None,
            password: None,
        })
        .await?;

        conn.ping().await?;

        Ok(())
    }

    #[runtime::test]
    async fn it_does_not_connect() -> Result<(), Error> {
        match Connection::establish(ConnectOptions {
            host: "127.0.0.1",
            port: 3306,
            user: Some("roote"),
            database: None,
            password: None,
        })
        .await {
            Ok(_) => Err(err_msg("Bad username still worked?")),
            Err(_) => Ok(())
        }
    }
}
