#[cfg(feature = "async")]
use std::future::Future;
use std::io;

use crate::Runtime;

// 's: stream
pub trait Stream<'s, Rt>: Send + Sync + Unpin
where
    Rt: Runtime,
{
    #[cfg(feature = "async")]
    type ReadFuture: 's + Future<Output = io::Result<usize>> + Send;

    #[cfg(feature = "async")]
    type WriteFuture: 's + Future<Output = io::Result<usize>> + Send;

    #[cfg(feature = "async")]
    #[doc(hidden)]
    fn read_async(&'s mut self, buf: &'s mut [u8]) -> Self::ReadFuture
    where
        Rt: crate::Async;

    #[cfg(feature = "async")]
    #[doc(hidden)]
    fn write_async(&'s mut self, buf: &'s [u8]) -> Self::WriteFuture
    where
        Rt: crate::Async;

    #[cfg(feature = "blocking")]
    #[doc(hidden)]
    fn read(&'s mut self, buf: &'s mut [u8]) -> io::Result<usize>
    where
        Rt: crate::blocking::Runtime;

    #[cfg(feature = "blocking")]
    #[doc(hidden)]
    fn write(&'s mut self, buf: &'s [u8]) -> io::Result<usize>
    where
        Rt: crate::blocking::Runtime;
}

#[cfg(not(any(
    feature = "async-std",
    feature = "actix",
    feature = "tokio",
    feature = "blocking"
)))]
impl<'s, Rt> Stream<'s, Rt> for ()
where
    Rt: Runtime,
{
    #[cfg(feature = "async")]
    type ReadFuture = futures_util::future::BoxFuture<'s, io::Result<usize>>;

    #[cfg(feature = "async")]
    type WriteFuture = futures_util::future::BoxFuture<'s, io::Result<usize>>;

    #[cfg(feature = "async")]
    #[doc(hidden)]
    #[allow(unused_variables)]
    fn read_async(&'s mut self, buf: &'s mut [u8]) -> Self::ReadFuture {
        // UNREACHABLE: where Self: Async
        unreachable!()
    }

    #[cfg(feature = "async")]
    #[doc(hidden)]
    #[allow(unused_variables)]
    fn write_async(&'s mut self, buf: &'s [u8]) -> Self::WriteFuture {
        // UNREACHABLE: where Self: Async
        unreachable!()
    }
}
