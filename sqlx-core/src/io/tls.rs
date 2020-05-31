use std::io;
use std::net::Shutdown;
use std::pin::Pin;
use std::task::{Context, Poll};

use crate::runtime::{AsyncRead, AsyncWrite, TcpStream};

use self::Inner::*;

pub struct MaybeTlsStream {
    inner: Inner,
}

enum Inner {
    NotTls(TcpStream),
    #[cfg(all(feature = "postgres", unix))]
    UnixStream(crate::runtime::UnixStream),
    #[cfg(feature = "tls")]
    Tls(async_native_tls::TlsStream<TcpStream>),
    #[cfg(feature = "tls")]
    Upgrading,
}

impl MaybeTlsStream {
    #[cfg(all(feature = "postgres", unix))]
    pub async fn connect_uds<S: AsRef<std::ffi::OsStr>>(p: S) -> crate::Result<Self> {
        let conn = crate::runtime::UnixStream::connect(p.as_ref()).await?;
        Ok(Self {
            inner: Inner::UnixStream(conn),
        })
    }
    pub async fn connect(host: &str, port: u16) -> crate::Result<Self> {
        let conn = TcpStream::connect((host, port)).await?;
        Ok(Self {
            inner: Inner::NotTls(conn),
        })
    }

    #[allow(dead_code)]
    pub fn is_tls(&self) -> bool {
        match self.inner {
            Inner::NotTls(_) => false,
            #[cfg(all(feature = "postgres", unix))]
            Inner::UnixStream(_) => false,
            #[cfg(feature = "tls")]
            Inner::Tls(_) => true,
            #[cfg(feature = "tls")]
            Inner::Upgrading => false,
        }
    }

    #[cfg(feature = "tls")]
    #[cfg_attr(docsrs, doc(cfg(feature = "tls")))]
    pub async fn upgrade(
        &mut self,
        host: &str,
        connector: async_native_tls::TlsConnector,
    ) -> crate::Result<()> {
        let conn = match std::mem::replace(&mut self.inner, Upgrading) {
            NotTls(conn) => conn,
            #[cfg(all(feature = "postgres", unix))]
            UnixStream(_) => {
                return Err(tls_err!("TLS is not supported with unix domain sockets").into())
            }
            Tls(_) => return Err(tls_err!("connection already upgraded").into()),
            Upgrading => return Err(tls_err!("connection already failed to upgrade").into()),
        };

        self.inner = Tls(connector.connect(host, conn).await?);

        Ok(())
    }

    pub fn shutdown(&self, how: Shutdown) -> io::Result<()> {
        match self.inner {
            NotTls(ref conn) => conn.shutdown(how),
            #[cfg(all(feature = "postgres", unix))]
            UnixStream(ref conn) => conn.shutdown(how),
            #[cfg(feature = "tls")]
            Tls(ref conn) => conn.get_ref().shutdown(how),
            #[cfg(feature = "tls")]
            // connection already closed
            Upgrading => Ok(()),
        }
    }
}

macro_rules! forward_pin (
    ($self:ident.$method:ident($($arg:ident),*)) => (
        match &mut $self.inner {
            NotTls(ref mut conn) => Pin::new(conn).$method($($arg),*),
            #[cfg(all(feature = "postgres", unix))]
            UnixStream(ref mut conn) => Pin::new(conn).$method($($arg),*),
            #[cfg(feature = "tls")]
            Tls(ref mut conn) => Pin::new(conn).$method($($arg),*),
            #[cfg(feature = "tls")]
            Upgrading => Err(io::Error::new(io::ErrorKind::Other, "connection broken; TLS upgrade failed")).into(),
        }
    )
);

impl AsyncRead for MaybeTlsStream {
    fn poll_read(
        mut self: Pin<&mut Self>,
        cx: &mut Context,
        buf: &mut [u8],
    ) -> Poll<io::Result<usize>> {
        forward_pin!(self.poll_read(cx, buf))
    }

    #[cfg(feature = "runtime-async-std")]
    fn poll_read_vectored(
        mut self: Pin<&mut Self>,
        cx: &mut Context,
        bufs: &mut [std::io::IoSliceMut],
    ) -> Poll<io::Result<usize>> {
        forward_pin!(self.poll_read_vectored(cx, bufs))
    }
}

impl AsyncWrite for MaybeTlsStream {
    fn poll_write(
        mut self: Pin<&mut Self>,
        cx: &mut Context,
        buf: &[u8],
    ) -> Poll<io::Result<usize>> {
        forward_pin!(self.poll_write(cx, buf))
    }

    fn poll_flush(mut self: Pin<&mut Self>, cx: &mut Context) -> Poll<io::Result<()>> {
        forward_pin!(self.poll_flush(cx))
    }

    #[cfg(feature = "runtime-async-std")]
    fn poll_close(mut self: Pin<&mut Self>, cx: &mut Context) -> Poll<io::Result<()>> {
        forward_pin!(self.poll_close(cx))
    }

    #[cfg(feature = "runtime-tokio")]
    fn poll_shutdown(mut self: Pin<&mut Self>, cx: &mut Context) -> Poll<io::Result<()>> {
        forward_pin!(self.poll_shutdown(cx))
    }

    #[cfg(feature = "runtime-async-std")]
    fn poll_write_vectored(
        mut self: Pin<&mut Self>,
        cx: &mut Context,
        bufs: &[std::io::IoSlice],
    ) -> Poll<io::Result<usize>> {
        forward_pin!(self.poll_write_vectored(cx, bufs))
    }
}
