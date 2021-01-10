use std::io;

use super::{Blocking, Runtime};

pub trait Connect<Rt: Runtime = Blocking>: crate::Connect<Rt> {
    fn connect(url: &str) -> crate::Result<Self>
    where
        Self: Sized,
        <Rt as crate::Runtime>::TcpStream: io::Read + io::Write;
}

// TODO: impl Connect for Pool { ... }
// TODO: impl Connect for PgConnection { ... }
