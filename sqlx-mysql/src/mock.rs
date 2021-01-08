use std::io;

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
        use futures_util::AsyncWriteExt;

        Box::pin(async move {
            self.write_all(&packet.len().to_le_bytes()[..3]).await?;
            self.write_all(&[seq]).await?;
            self.write_all(packet).await
        })
    }

    #[cfg(feature = "async")]
    fn read_exact_async(
        &mut self,
        n: usize,
    ) -> futures_util::future::BoxFuture<'_, io::Result<Vec<u8>>> {
        use futures_util::AsyncReadExt;

        Box::pin(async move {
            let mut buf = vec![0; n];
            let read = self.read(&mut buf).await?;
            buf.truncate(read);

            Ok(buf)
        })
    }

    #[cfg(feature = "async")]
    fn read_all_async(&mut self) -> futures_util::future::BoxFuture<'_, io::Result<Vec<u8>>> {
        use futures_util::AsyncReadExt;

        Box::pin(async move {
            let mut buf = vec![0; 1024];
            let read = self.read(&mut buf).await?;
            buf.truncate(read);

            Ok(buf)
        })
    }
}
