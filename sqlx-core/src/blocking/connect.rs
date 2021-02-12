use super::Runtime;

pub trait Connect<Rt>: crate::Connect<Rt>
where
    Rt: Runtime,
{
    fn connect(url: &str) -> crate::Result<Self>
    where
        Self: Sized,
    {
        <Self as Connect<Rt>>::connect_with(&url.parse::<Self::Options>()?)
    }

    fn connect_with(options: &Self::Options) -> crate::Result<Self>
    where
        Self: Sized;
}
