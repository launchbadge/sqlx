use std::io;

use super::{Blocking, Runtime};

pub trait Close<Rt: Runtime = Blocking>: crate::Close<Rt> {
    fn close(self) -> crate::Result<()>
    where
        <Rt as crate::Runtime>::TcpStream: io::Read + io::Write;
}

// TODO: impl Close for Pool { ... }
// TODO: impl<C: Connection> Close for C { ... }
