use crate::connection::ConnectOptions;
use crate::error::Error;
use crate::executor::Executor;
use crate::sqlite::connection::establish::establish;
use crate::sqlite::{SqliteConnectOptions, SqliteConnection};
use futures_core::future::BoxFuture;
use log::LevelFilter;
use std::time::Duration;

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

    fn log_statements(&mut self, level: LevelFilter) -> &mut Self {
        self.log_settings.log_statements(level);
        self
    }

    fn log_slow_statements(&mut self, level: LevelFilter, duration: Duration) -> &mut Self {
        self.log_settings.log_slow_statements(level, duration);
        self
    }
}
