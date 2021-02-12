use crate::{ConnectOptions, Runtime};

pub trait Connect<Rt>
where
    Rt: Runtime,
{
    type Options: ConnectOptions;

    #[cfg(feature = "async")]
    fn connect(url: &str) -> futures_util::future::BoxFuture<'_, crate::Result<Self>>
    where
        Self: Sized,
        Rt: crate::Async,
    {
        let options = url.parse::<Self::Options>();
        Box::pin(async move { Self::connect_with(&options?).await })
    }

    #[cfg(feature = "async")]
    fn connect_with(
        options: &Self::Options,
    ) -> futures_util::future::BoxFuture<'_, crate::Result<Self>>
    where
        Self: Sized,
        Rt: crate::Async;
}
