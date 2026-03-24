use crate::connection::ConnectOptions;
use crate::error::Error;
use crate::{MssqlConnectOptions, MssqlConnection};
use log::LevelFilter;
use sqlx_core::Url;
use std::time::Duration;

impl ConnectOptions for MssqlConnectOptions {
    type Connection = MssqlConnection;

    fn from_url(url: &Url) -> Result<Self, Error> {
        Self::parse_from_url(url)
    }

    fn to_url_lossy(&self) -> Url {
        self.build_url()
            .expect("BUG: MssqlConnectOptions generated an un-parseable URL")
    }

    async fn connect(&self) -> Result<Self::Connection, Error>
    where
        Self::Connection: Sized,
    {
        MssqlConnection::establish(self).await
    }

    fn log_statements(mut self, level: LevelFilter) -> Self {
        self.log_settings.log_statements(level);
        self
    }

    fn log_slow_statements(mut self, level: LevelFilter, duration: Duration) -> Self {
        self.log_settings.log_slow_statements(level, duration);
        self
    }
}
