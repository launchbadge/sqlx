use super::Runtime;

pub trait Close<Rt>: crate::Close<Rt>
where
    Rt: Runtime,
{
    fn close(self) -> crate::Result<()>;
}

// TODO: impl Close for Pool { ... }
// TODO: impl<C: Connection> Close for C { ... }
