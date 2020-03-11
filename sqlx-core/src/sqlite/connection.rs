use std::convert::TryInto;

use futures_core::future::BoxFuture;

use crate::connection::{Connect, Connection};
use crate::url::Url;

pub struct SqliteConnection {}

impl Connect for SqliteConnection {
    fn connect<T>(url: T) -> BoxFuture<'static, crate::Result<SqliteConnection>>
    where
        T: TryInto<Url, Error = crate::Error>,
        Self: Sized,
    {
        // Box::pin(SqliteConnection::new(url.try_into()))
        todo!()
    }
}

impl Connection for SqliteConnection {
    fn close(self) -> BoxFuture<'static, crate::Result<()>> {
        // Box::pin(terminate(self.stream))
        todo!()
    }

    fn ping(&mut self) -> BoxFuture<crate::Result<()>> {
        //Box::pin(Executor::execute(self, "SELECT 1").map_ok(|_| ()))
        todo!()
    }
}
