use crate::aurora::connection::AuroraConnection;
use crate::aurora::options::AuroraConnectOptions;
use crate::connection::ConnectOptions;
use crate::error::Error;

use futures_core::future::BoxFuture;
use log::LevelFilter;
use std::time::Duration;

impl ConnectOptions for AuroraConnectOptions {
    type Connection = AuroraConnection;

    fn connect(&self) -> BoxFuture<'_, Result<Self::Connection, Error>>
    where
        Self::Connection: Sized,
    {
        Box::pin(async move { AuroraConnection::new(self) })
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
