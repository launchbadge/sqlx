use crate::Runtime;

// for<'a> &'a mut Rt::TcpStream: crate::io::Stream<'a>,
pub trait Close<Rt>
where
    Rt: Runtime,
{
    #[cfg(feature = "async")]
    fn close(self) -> futures_util::future::BoxFuture<'static, crate::Result<()>>
    where
        Rt: crate::Async,
        for<'s> <Rt as Runtime>::TcpStream: crate::io::Stream<'s, Rt>;
}

// TODO: impl Close for Pool { ... }
// TODO: impl<C: Connection> Close for C { ... }
