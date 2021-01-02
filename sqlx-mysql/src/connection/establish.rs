use bytes::{buf::Chain, Buf, Bytes};
use futures_io::{AsyncRead, AsyncWrite};
use sqlx_core::io::{Deserialize, Serialize};
use sqlx_core::{AsyncRuntime, Error, Result, Runtime};

use crate::protocol::{Capabilities, ErrPacket, Handshake, HandshakeResponse, OkPacket};
use crate::{MySqlConnectOptions, MySqlConnection, MySqlDatabaseError};

// https://dev.mysql.com/doc/internals/en/connection-phase.html

// the connection phase (establish) performs these tasks:
//  - exchange the capabilities of client and server
//  - setup SSL communication channel if requested
//  - authenticate the client against the server

// the server may immediately send an ERR packet and finish the handshake
// or send a [InitialHandshake]

fn make_auth_response(
    auth_plugin_name: Option<&str>,
    username: &str,
    password: Option<&str>,
    nonce: &Chain<Bytes, Bytes>,
) -> Vec<u8> {
    vec![]
}

fn make_handshake_response<Rt: Runtime>(options: &MySqlConnectOptions<Rt>) -> HandshakeResponse<'_> {
    HandshakeResponse {
        auth_plugin_name: None,
        auth_response: None,
        charset: 45, // [utf8mb4]
        database: options.get_database(),
        max_packet_size: 1024,
        username: options.get_username(),
    }
}

impl<Rt> MySqlConnection<Rt>
where
    Rt: AsyncRuntime,
    <Rt as Runtime>::TcpStream: Unpin + AsyncWrite + AsyncRead,
{
    fn recv_handshake(&mut self, handshake: &Handshake) {
        self.capabilities &= handshake.capabilities;
        self.connection_id = handshake.connection_id;
    }

    pub(crate) async fn establish_async(options: &MySqlConnectOptions<Rt>) -> Result<Self> {
        let stream = Rt::connect_tcp(options.get_host(), options.get_port()).await?;
        let mut self_ = Self::new(stream);

        let handshake = self_.read_packet_async().await?;
        self_.recv_handshake(&handshake);

        self_.write_packet(make_handshake_response(options))?;

        self_.stream.flush_async().await?;

        let _ok: OkPacket = self_.read_packet_async().await?;

        Ok(self_)
    }

    fn write_packet<'ser, T>(&'ser mut self, packet: T) -> Result<()>
    where
        T: Serialize<'ser, Capabilities>,
    {
        let mut wbuf = Vec::<u8>::with_capacity(1024);

        packet.serialize_with(&mut wbuf, self.capabilities)?;

        self.sequence_id = self.sequence_id.wrapping_add(1);

        self.stream.reserve(wbuf.len() + 4);
        self.stream.write(&(wbuf.len() as u32).to_le_bytes()[..3]);
        self.stream.write(&[self.sequence_id]);
        self.stream.write(&wbuf);

        Ok(())
    }

    async fn read_packet_async<'de, T>(&'de mut self) -> Result<T>
    where
        T: Deserialize<'de, Capabilities>,
    {
        // https://dev.mysql.com/doc/internals/en/mysql-packet.html
        self.stream.read_async(4).await?;

        let payload_len: usize = self.stream.get(0, 3).get_int_le(3) as usize;

        // FIXME: handle split packets
        assert_ne!(payload_len, 0xFF_FF_FF);

        self.sequence_id = self.stream.get(3, 1).get_u8();

        self.stream.read_async(4 + payload_len).await?;

        self.stream.consume(4);
        let payload = self.stream.take(payload_len);

        if payload[0] == 0xff {
            // if the first byte of the payload is 0xFF and the payload is an ERR packet
            let err = ErrPacket::deserialize_with(payload, self.capabilities)?;
            return Err(Error::Connect(Box::new(MySqlDatabaseError(err))));
        }

        T::deserialize_with(payload, self.capabilities)
    }
}
