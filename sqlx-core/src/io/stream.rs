use crate::Runtime;

// 's: stream
pub trait Stream<'s, Rt: Runtime>: Send + Sync + Unpin {
    #[cfg(feature = "async")]
    type ReadFuture: 's + std::future::Future<Output = std::io::Result<usize>> + Send;

    #[cfg(feature = "async")]
    type WriteFuture: 's + std::future::Future<Output = std::io::Result<usize>> + Send;

    #[cfg(feature = "async")]
    #[doc(hidden)]
    fn read_async(&'s mut self, buf: &'s mut [u8]) -> Self::ReadFuture;

    #[cfg(feature = "async")]
    #[doc(hidden)]
    fn write_async(&'s mut self, buf: &'s [u8]) -> Self::WriteFuture;
}
