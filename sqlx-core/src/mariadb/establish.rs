use crate::{
    mariadb::{
        connection::MariaDb,
        protocol::{Capabilities, HandshakeResponsePacket, InitialHandshakePacket},
    },
    Result,
};
use url::Url;

pub(crate) async fn establish(conn: &mut MariaDb, url: &Url) -> Result<()> {
    let initial = InitialHandshakePacket::decode(conn.receive().await?)?;

    // TODO: Capabilities::SECURE_CONNECTION
    // TODO: Capabilities::CONNECT_ATTRS
    // TODO: Capabilities::PLUGIN_AUTH
    // TODO: Capabilities::PLUGIN_AUTH_LENENC_CLIENT_DATA
    // TODO: Capabilities::TRANSACTIONS
    // TODO: Capabilities::CLIENT_DEPRECATE_EOF
    // TODO?: Capabilities::CLIENT_SESSION_TRACK
    let capabilities = Capabilities::CLIENT_PROTOCOL_41 | Capabilities::CONNECT_WITH_DB;

    let response = HandshakeResponsePacket {
        // TODO: Find a good value for [max_packet_size]
        capabilities,
        max_packet_size: 1024,
        client_collation: 192, // utf8_unicode_ci
        username: url.username(),
        database: &url.path()[1..],
        auth_data: None,
        auth_plugin_name: None,
        connection_attrs: &[],
    };

    // The AND between our supported capabilities and the servers' is
    // what we can use so remember it on the connection
    conn.capabilities = capabilities & initial.capabilities;

    conn.write(response);
    conn.stream.flush().await?;

    let _ = conn.receive_ok_or_err().await?;

    // TODO: If CONNECT_WITH_DB is not supported we need to send an InitDb command just after establish

    Ok(())
}
