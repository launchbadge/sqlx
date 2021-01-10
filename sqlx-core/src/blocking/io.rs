use std::io;

use crate::Runtime;

// 's: stream
pub trait Stream<'s, Rt>: crate::io::Stream<'s, Rt>
where
    Rt: Runtime,
{
    #[doc(hidden)]
    fn read(&'s mut self, buf: &'s mut [u8]) -> io::Result<usize>;

    #[doc(hidden)]
    fn write(&'s mut self, buf: &'s [u8]) -> io::Result<usize>;
}
