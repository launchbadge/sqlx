use crate::connection::ConnectOptions;
use crate::error::Error;
use crate::executor::Executor;
use crate::sqlite::{SqliteConnectOptions, SqliteConnection};
use futures_core::future::BoxFuture;
use log::LevelFilter;
use std::fmt::Write;
use std::time::Duration;

impl ConnectOptions for SqliteConnectOptions {
    type Connection = SqliteConnection;

    fn connect(&self) -> BoxFuture<'_, Result<Self::Connection, Error>>
    where
        Self::Connection: Sized,
    {
        Box::pin(async move {
            let mut conn = SqliteConnection::establish(self).await?;

            // send an initial sql statement comprised of options
            let mut init = String::new();

            // This is a special case for sqlcipher. When the `key` pragma
            // is set, we have to make sure it's executed first in order.
            if let Some(pragma_key_password) = self.pragmas.get("key") {
                write!(init, "PRAGMA key = {}; ", pragma_key_password).ok();
            }

            for (key, value) in &self.pragmas {
                // Since we've already written the possible `key` pragma
                // above, we shall skip it now.
                if key == "key" {
                    continue;
                }
                write!(init, "PRAGMA {} = {}; ", key, value).ok();
            }

            conn.execute(&*init).await?;

            if !self.collations.is_empty() {
                let mut locked = conn.lock_handle().await?;

                for collation in &self.collations {
                    collation.create(&mut locked.guard.handle)?;
                }
            }

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
