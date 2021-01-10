use sqlx_core::{Result, Runtime};

use crate::protocol::Quit;

impl<Rt> super::MySqlConnection<Rt>
where
    Rt: Runtime,
{
    #[cfg(feature = "async")]
    pub(crate) async fn close_async(mut self) -> Result<()>
    where
        Rt: sqlx_core::Async,
        for<'s> <Rt as Runtime>::TcpStream: sqlx_core::io::Stream<'s, Rt>,
    {
        self.write_packet(&Quit)?;
        self.stream.flush_async().await?;

        Ok(())
    }

    #[cfg(feature = "blocking")]
    pub(crate) fn close(mut self) -> Result<()>
    where
        Rt: sqlx_core::blocking::Runtime,
        for<'s> <Rt as Runtime>::TcpStream: sqlx_core::blocking::io::Stream<'s, Rt>,
    {
        self.write_packet(&Quit)?;
        self.stream.flush()?;

        Ok(())
    }
}
