use super::{io::Stream, Runtime};

pub trait Close<Rt>: crate::Close<Rt>
where
    Rt: Runtime,
{
    fn close(self) -> crate::Result<()>
    where
        for<'s> <Rt as crate::Runtime>::TcpStream: Stream<'s, Rt>;
}

// TODO: impl Close for Pool { ... }
// TODO: impl<C: Connection> Close for C { ... }
