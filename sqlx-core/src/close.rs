use crate::{DefaultRuntime, Runtime};

pub trait Close<Rt: Runtime = DefaultRuntime> {
    #[cfg(feature = "async")]
    fn close(self) -> futures_util::future::BoxFuture<'static, crate::Result<()>>
    where
        Rt: crate::AsyncRuntime,
        <Rt as Runtime>::TcpStream: futures_io::AsyncRead + futures_io::AsyncWrite + Unpin;
}

// TODO: impl Close for Pool { ... }
// TODO: impl<C: Connection> Close for C { ... }
