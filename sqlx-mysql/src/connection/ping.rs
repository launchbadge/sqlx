use sqlx_core::{Result, Runtime};

use crate::protocol::{OkPacket, Ping};

// PING is very simple
// send the COM_PING packet
// should receive an OK

impl<Rt> super::MySqlConnection<Rt>
where
    Rt: Runtime,
{
    #[cfg(feature = "async")]
    pub(crate) async fn ping_async(&mut self) -> Result<()>
    where
        Rt: sqlx_core::AsyncRuntime,
        <Rt as Runtime>::TcpStream: futures_io::AsyncWrite + futures_io::AsyncRead + Unpin,
    {
        self.write_packet(&Ping)?;

        let _ok: OkPacket = self.read_packet_async().await?;

        Ok(())
    }

    #[cfg(feature = "blocking")]
    pub(crate) fn ping(&mut self) -> Result<()>
    where
        <Rt as Runtime>::TcpStream: std::io::Write + std::io::Read,
    {
        self.write_packet(&Ping)?;

        let _ok: OkPacket = self.read_packet()?;

        Ok(())
    }
}
