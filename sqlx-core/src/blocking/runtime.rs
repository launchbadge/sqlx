use std::io;
use std::net::TcpStream;

/// Describes a set of types and functions used to open and manage
/// resources within SQLx.
///
/// For detailed information, refer to the asynchronous version of
/// this: [`Runtime`][crate::Runtime].
///
pub trait Runtime: crate::Runtime {
    /// Opens a TCP connection to a remote host at the specified port.
    fn connect_tcp(host: &str, port: u16) -> io::Result<Self::TcpStream>;
}

/// Uses the `std::net` primitives to implement a blocking runtime for SQLx.
#[derive(Debug)]
pub struct Blocking;

impl crate::Runtime for Blocking {
    type TcpStream = TcpStream;
}

impl Runtime for Blocking {
    fn connect_tcp(host: &str, port: u16) -> io::Result<Self::TcpStream> {
        TcpStream::connect((host, port))
    }
}
