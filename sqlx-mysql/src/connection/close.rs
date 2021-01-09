use sqlx_core::{Result, Runtime};

use crate::protocol::Quit;

impl<Rt> super::MySqlConnection<Rt>
where
    Rt: Runtime,
{
    #[cfg(feature = "async")]
    pub(crate) async fn close_async(mut self) -> Result<()>
    where
        Rt: sqlx_core::AsyncRuntime,
        <Rt as Runtime>::TcpStream: futures_io::AsyncWrite + futures_io::AsyncRead + Unpin,
    {
        self.write_packet(&Quit)?;
        self.stream.flush_async().await?;

        Ok(())
    }

    #[cfg(feature = "blocking")]
    pub(crate) fn close(mut self) -> Result<()>
    where
        <Rt as Runtime>::TcpStream: std::io::Write + std::io::Read,
    {
        self.write_packet(&Quit)?;
        self.stream.flush()?;

        Ok(())
    }
}
