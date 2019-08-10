use crate::backend::Backend;
use futures::future::BoxFuture;
use std::{
    io,
    ops::{Deref, DerefMut},
};
use url::Url;

// TODO: Re-implement and forward to Raw instead of using Deref

pub trait RawConnection {
    fn establish(url: &Url) -> BoxFuture<io::Result<Self>>
    where
        Self: Sized;
}

pub struct Connection<B>
where
    B: Backend,
{
    pub(crate) inner: B::RawConnection,
}

impl<B> Connection<B>
where
    B: Backend,
{
    #[inline]
    pub async fn establish(url: &str) -> io::Result<Self> {
        // TODO: Handle url parse errors
        let url = Url::parse(url).unwrap();

        Ok(Self {
            inner: B::RawConnection::establish(&url).await?,
        })
    }
}

impl<B> Deref for Connection<B>
where
    B: Backend,
{
    type Target = B::RawConnection;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl<B> DerefMut for Connection<B>
where
    B: Backend,
{
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}
