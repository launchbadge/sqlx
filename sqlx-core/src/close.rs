use crate::Runtime;

pub trait Close<Rt>
where
    Rt: Runtime,
{
    #[cfg(feature = "async")]
    fn close(self) -> futures_util::future::BoxFuture<'static, crate::Result<()>>
    where
        Rt: crate::Async;
}

// TODO: impl Close for Pool { ... }
// TODO: impl<C: Connection> Close for C { ... }
