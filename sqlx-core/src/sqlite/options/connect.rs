use crate::connection::ConnectOptions;
use crate::error::Error;
use crate::executor::Executor;
use crate::sqlite::connection::establish::establish;
use crate::sqlite::{SqliteConnectOptions, SqliteConnection};
use futures_core::future::BoxFuture;

impl ConnectOptions for SqliteConnectOptions {
    type Connection = SqliteConnection;

    fn connect(&self) -> BoxFuture<'_, Result<Self::Connection, Error>>
    where
        Self::Connection: Sized,
    {
        Box::pin(async move {
            let mut conn = establish(self).await?;

            // send an initial sql statement comprised of options
            let init = format!(
                "PRAGMA journal_mode = {}; PRAGMA foreign_keys = {};",
                self.journal_mode.as_str(),
                if self.foreign_keys { "ON" } else { "OFF" }
            );

            conn.execute(&*init).await?;

            Ok(conn)
        })
    }
}
