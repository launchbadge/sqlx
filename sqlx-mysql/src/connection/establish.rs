use bytes::Buf;
use futures_io::{AsyncRead, AsyncWrite};
use sqlx_core::io::{BufStream, Deserialize};
use sqlx_core::{AsyncRuntime, Result, Runtime};

use crate::protocol::Handshake;
use crate::{MySqlConnectOptions, MySqlConnection};

// https://dev.mysql.com/doc/internals/en/connection-phase.html

// the connection phase (establish) performs these tasks:
//  - exchange the capabilities of client and server
//  - setup SSL communication channel if requested
//  - authenticate the client against the server

// the server may immediately send an ERR packet and finish the handshake
// or send a [InitialHandshake]

impl<Rt> MySqlConnection<Rt>
where
    Rt: AsyncRuntime,
    <Rt as Runtime>::TcpStream: Unpin + AsyncWrite + AsyncRead,
{
    pub(crate) async fn establish_async(options: &MySqlConnectOptions<Rt>) -> Result<Self> {
        let stream = Rt::connect_tcp(options.get_host(), options.get_port()).await?;
        let mut self_ = Self::new(stream);

        // FIXME: Handle potential ERR packet here
        let handshake = self_.read_packet_async::<Handshake>().await?;
        println!("{:#?}", handshake);

        Ok(self_)
    }

    async fn read_packet_async<'de, T>(&'de mut self) -> Result<T>
    where
        T: Deserialize<'de>,
    {
        // https://dev.mysql.com/doc/internals/en/mysql-packet.html
        self.stream.read_async(4).await?;

        let payload_len: usize = self.stream.get(0, 3).get_int_le(3) as usize;

        // FIXME: handle split packets
        assert_ne!(payload_len, 0xFF_FF_FF);

        let _seq_no = self.stream.get(3, 1).get_i8();

        self.stream.read_async(4 + payload_len).await?;

        self.stream.consume(4);
        let payload = self.stream.take(payload_len);

        T::deserialize(payload)
    }
}
