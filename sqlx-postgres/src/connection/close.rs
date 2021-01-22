use sqlx_core::{io::Stream, Result, Runtime};

use crate::protocol::Terminate;

impl<Rt> super::PostgresConnection<Rt>
where
    Rt: Runtime,
{
    #[cfg(feature = "async")]
    pub(crate) async fn close_async(mut self) -> Result<()>
    where
        Rt: sqlx_core::Async,
    {
        self.write_packet(&Terminate)?;
        self.stream.flush_async().await?;
        self.stream.shutdown_async().await?;

        Ok(())
    }

    #[cfg(feature = "blocking")]
    pub(crate) fn close(mut self) -> Result<()>
    where
        Rt: sqlx_core::blocking::Runtime,
    {
        self.write_packet(&Terminate)?;
        self.stream.flush()?;
        self.stream.shutdown()?;

        Ok(())
    }
}
