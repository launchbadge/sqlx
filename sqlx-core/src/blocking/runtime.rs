use std::io;
use std::net::TcpStream;

/// Describes a set of types and functions used to open and manage
/// resources within SQLx using blocking I/O.
///
pub trait Runtime {
    type TcpStream;

    /// Opens a TCP connection to a remote host at the specified port.
    fn connect_tcp(host: &str, port: u16) -> io::Result<Self::TcpStream>;
}

/// Uses the `std::net` primitives to implement a blocking runtime for SQLx.
#[cfg_attr(doc_cfg, doc(cfg(feature = "blocking")))]
#[derive(Debug)]
pub struct Blocking;

impl Runtime for Blocking {
    type TcpStream = TcpStream;

    fn connect_tcp(host: &str, port: u16) -> io::Result<Self::TcpStream> {
        TcpStream::connect((host, port))
    }
}
