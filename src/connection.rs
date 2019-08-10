pub(crate) use self::internal::ConnectionAssocQuery;
use crate::{backend::Backend, Query};
use futures::future::BoxFuture;
use std::io;

mod internal {
    pub trait ConnectionAssocQuery<'c, 'q> {
        type Query: super::Query<'c, 'q>;
    }
}

pub trait Connection: for<'c, 'q> ConnectionAssocQuery<'c, 'q> {
    type Backend: Backend;

    fn establish(url: &str) -> BoxFuture<io::Result<Self>>
    where
        Self: Sized;

    fn prepare<'c, 'q>(
        &'c mut self,
        query: &'q str,
    ) -> <Self as ConnectionAssocQuery<'c, 'q>>::Query;
}
