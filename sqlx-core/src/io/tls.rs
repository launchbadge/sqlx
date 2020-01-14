use std::io::{IoSlice, IoSliceMut};
use std::pin::Pin;
use std::task::{Context, Poll};

use async_std::io::{self, Read, Write};
use async_std::net::{Shutdown, TcpStream};

use crate::url::Url;

use self::Inner::*;

pub struct MaybeTlsStream {
    inner: Inner,
}

enum Inner {
    NotTls(TcpStream),
    #[cfg(feature = "tls")]
    Tls(async_native_tls::TlsStream<TcpStream>),
    #[cfg(feature = "tls")]
    Upgrading,
}

impl MaybeTlsStream {
    pub async fn connect(url: &Url, default_port: u16) -> crate::Result<Self> {
        let conn = TcpStream::connect((url.host(), url.port(default_port))).await?;
        Ok(Self {
            inner: Inner::NotTls(conn),
        })
    }

    #[allow(dead_code)]
    pub fn is_tls(&self) -> bool {
        match self.inner {
            Inner::NotTls(_) => false,
            #[cfg(feature = "tls")]
            Inner::Tls(_) => true,
            #[cfg(feature = "tls")]
            Inner::Upgrading => false,
        }
    }

    #[cfg(feature = "tls")]
    pub async fn upgrade(
        &mut self,
        url: &Url,
        connector: async_native_tls::TlsConnector,
    ) -> crate::Result<()> {
        let conn = match std::mem::replace(&mut self.inner, Upgrading) {
            NotTls(conn) => conn,
            Tls(_) => return Err(tls_err!("connection already upgraded").into()),
            Upgrading => return Err(tls_err!("connection already failed to upgrade").into()),
        };

        self.inner = Tls(connector.connect(url.host(), conn).await?);

        Ok(())
    }

    pub fn shutdown(&self, how: Shutdown) -> io::Result<()> {
        match self.inner {
            NotTls(ref conn) => conn.shutdown(how),
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
            #[cfg(feature = "tls")]
            Tls(ref mut conn) => Pin::new(conn).$method($($arg),*),
            #[cfg(feature = "tls")]
            Upgrading => Err(io::Error::new(io::ErrorKind::Other, "connection broken; TLS upgrade failed")).into(),
        }
    )
);

impl Read for MaybeTlsStream {
    fn poll_read(
        mut self: Pin<&mut Self>,
        cx: &mut Context,
        buf: &mut [u8],
    ) -> Poll<io::Result<usize>> {
        forward_pin!(self.poll_read(cx, buf))
    }

    fn poll_read_vectored(
        mut self: Pin<&mut Self>,
        cx: &mut Context,
        bufs: &mut [IoSliceMut],
    ) -> Poll<io::Result<usize>> {
        forward_pin!(self.poll_read_vectored(cx, bufs))
    }
}

impl Write for MaybeTlsStream {
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

    fn poll_close(mut self: Pin<&mut Self>, cx: &mut Context) -> Poll<io::Result<()>> {
        forward_pin!(self.poll_close(cx))
    }

    fn poll_write_vectored(
        mut self: Pin<&mut Self>,
        cx: &mut Context,
        bufs: &[IoSlice],
    ) -> Poll<io::Result<usize>> {
        forward_pin!(self.poll_write_vectored(cx, bufs))
    }
}
