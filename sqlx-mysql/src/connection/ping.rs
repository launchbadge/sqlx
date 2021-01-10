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
        Rt: sqlx_core::Async,
        for<'s> Rt::TcpStream: sqlx_core::io::Stream<'s, Rt>,
    {
        self.write_packet(&Ping)?;

        let _ok: OkPacket = self.read_packet_async().await?;

        Ok(())
    }

    #[cfg(feature = "blocking")]
    pub(crate) fn ping(&mut self) -> Result<()>
    where
        for<'s> Rt::TcpStream: sqlx_core::blocking::io::Stream<'s, Rt>,
    {
        self.write_packet(&Ping)?;

        let _ok: OkPacket = self.read_packet()?;

        Ok(())
    }
}
