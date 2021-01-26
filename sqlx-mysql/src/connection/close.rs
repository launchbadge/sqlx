use sqlx_core::{io::Stream, Result, Runtime};

use crate::protocol::Quit;

impl<Rt: Runtime> super::MySqlConnection<Rt> {
    #[cfg(feature = "async")]
    pub(crate) async fn close_async(mut self) -> Result<()>
    where
        Rt: sqlx_core::Async,
    {
        self.stream.write_packet(&Quit)?;
        self.stream.flush_async().await?;
        self.stream.shutdown_async().await?;

        Ok(())
    }

    #[cfg(feature = "blocking")]
    pub(crate) fn close_blocking(mut self) -> Result<()>
    where
        Rt: sqlx_core::blocking::Runtime,
    {
        self.stream.write_packet(&Quit)?;
        self.stream.flush()?;
        self.stream.shutdown()?;

        Ok(())
    }
}
