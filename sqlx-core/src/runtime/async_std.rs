use std::io;

use async_std::{net::TcpStream, task::block_on};
use futures_util::{future::BoxFuture, FutureExt};

use crate::{AsyncRuntime, Runtime};

/// [`async-std`](async_std) implementation of [`Runtime`].
#[cfg_attr(doc_cfg, doc(cfg(feature = "async-std")))]
#[derive(Debug)]
pub struct AsyncStd;

impl Runtime for AsyncStd {
    type TcpStream = TcpStream;
}

impl AsyncRuntime for AsyncStd {
    fn connect_tcp(host: &str, port: u16) -> BoxFuture<'_, io::Result<Self::TcpStream>> {
        TcpStream::connect((host, port)).boxed()
    }
}

#[cfg(feature = "blocking")]
impl crate::blocking::Runtime for AsyncStd {
    fn connect_tcp(host: &str, port: u16) -> io::Result<Self::TcpStream> {
        block_on(<AsyncStd as AsyncRuntime>::connect_tcp(host, port))
    }
}
