use super::{io::Stream, Runtime};

pub trait Connect<Rt>: crate::Connect<Rt>
where
    Rt: Runtime,
{
    fn connect(url: &str) -> crate::Result<Self>
    where
        Self: Sized,
        for<'s> <Rt as crate::Runtime>::TcpStream: Stream<'s, Rt>;
}

// TODO: impl Connect for Pool { ... }
// TODO: impl Connect for PgConnection { ... }
