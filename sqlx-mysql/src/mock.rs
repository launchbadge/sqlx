use std::io;

use sqlx_core::io::Stream;
use sqlx_core::mock::MockStream;

pub(crate) trait MySqlMockStreamExt {
    #[cfg(feature = "async")]
    fn write_packet_async<'x>(
        &'x mut self,
        seq: u8,
        packet: &'x [u8],
    ) -> futures_util::future::BoxFuture<'x, io::Result<()>>;

    #[cfg(feature = "async")]
    fn read_exact_async(
        &mut self,
        n: usize,
    ) -> futures_util::future::BoxFuture<'_, io::Result<Vec<u8>>>;

    #[cfg(feature = "async")]
    fn read_all_async(&mut self) -> futures_util::future::BoxFuture<'_, io::Result<Vec<u8>>>;
}

impl MySqlMockStreamExt for MockStream {
    #[cfg(feature = "async")]
    fn write_packet_async<'x>(
        &'x mut self,
        seq: u8,
        packet: &'x [u8],
    ) -> futures_util::future::BoxFuture<'x, io::Result<()>> {
        Box::pin(async move {
            self.write_async(&packet.len().to_le_bytes()[..3]).await?;
            self.write_async(&[seq]).await?;
            self.write_async(packet).await?;

            Ok(())
        })
    }

    #[cfg(feature = "async")]
    fn read_exact_async(
        &mut self,
        n: usize,
    ) -> futures_util::future::BoxFuture<'_, io::Result<Vec<u8>>> {
        Box::pin(async move {
            let mut buf = vec![0; n];
            let read = self.read_async(&mut buf).await?;
            buf.truncate(read);

            Ok(buf)
        })
    }

    #[cfg(feature = "async")]
    fn read_all_async(&mut self) -> futures_util::future::BoxFuture<'_, io::Result<Vec<u8>>> {
        Box::pin(async move {
            let mut buf = vec![0; 1024];
            let read = self.read_async(&mut buf).await?;
            buf.truncate(read);

            Ok(buf)
        })
    }
}
