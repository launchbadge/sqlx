use crate::{backend::Backend, ConnectOptions};
use futures::future::BoxFuture;
use std::io;

pub trait RawConnection {
    fn establish(options: ConnectOptions<'_>) -> BoxFuture<io::Result<Self>>
    where
        Self: Sized;
}
