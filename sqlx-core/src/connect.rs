use crate::{ConnectOptions, DefaultRuntime, Runtime};

pub trait Connect<Rt: Runtime = DefaultRuntime> {
    type Options: ConnectOptions<Rt, Connection = Self>;

    #[cfg(feature = "async")]
    fn connect(url: &str) -> futures_util::future::BoxFuture<'_, crate::Result<Self>>
    where
        Self: Sized,
        Rt: crate::AsyncRuntime,
        <Rt as Runtime>::TcpStream: futures_io::AsyncRead + futures_io::AsyncWrite + Unpin;
}

// TODO: impl Connect for Pool { ... }
// TODO: impl Connect for PgConnection { ... }
