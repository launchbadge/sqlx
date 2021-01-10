use std::io;

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
