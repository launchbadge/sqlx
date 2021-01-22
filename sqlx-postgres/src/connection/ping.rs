use sqlx_core::{Result, Runtime};

impl<Rt> super::PostgresConnection<Rt>
where
    Rt: Runtime,
{
    #[cfg(feature = "async")]
    pub(crate) async fn ping_async(&mut self) -> Result<()>
    where
        Rt: sqlx_core::Async,
    {
        todo!();
    }

    #[cfg(feature = "blocking")]
    pub(crate) fn ping(&mut self) -> Result<()>
    where
        Rt: sqlx_core::blocking::Runtime,
    {
        todo!();
    }
}
