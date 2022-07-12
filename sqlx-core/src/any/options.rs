use crate::any::AnyConnection;
use crate::connection::ConnectOptions;
use crate::error::Error;
use futures_core::future::BoxFuture;
use log::LevelFilter;
use std::str::FromStr;
use std::time::Duration;

#[cfg(feature = "postgres")]
use crate::postgres::PgConnectOptions;

#[cfg(feature = "mysql")]
use crate::mysql::MySqlConnectOptions;

#[cfg(feature = "sqlite")]
use crate::sqlite::SqliteConnectOptions;

use crate::any::kind::AnyKind;
#[cfg(feature = "mssql")]
use crate::mssql::MssqlConnectOptions;

/// Opaque options for connecting to a database. These may only be constructed by parsing from
/// a connection url.
///
/// ```text
/// postgres://postgres:password@localhost/database
/// mysql://root:password@localhost/database
/// ```
#[derive(Debug, Clone)]
pub struct AnyConnectOptions(pub(crate) AnyConnectOptionsKind);

impl AnyConnectOptions {
    pub fn kind(&self) -> AnyKind {
        match &self.0 {
            #[cfg(feature = "postgres")]
            AnyConnectOptionsKind::Postgres(_) => AnyKind::Postgres,

            #[cfg(feature = "mysql")]
            AnyConnectOptionsKind::MySql(_) => AnyKind::MySql,

            #[cfg(feature = "sqlite")]
            AnyConnectOptionsKind::Sqlite(_) => AnyKind::Sqlite,

            #[cfg(feature = "mssql")]
            AnyConnectOptionsKind::Mssql(_) => AnyKind::Mssql,
        }
    }
}

macro_rules! try_from_any_connect_options_to {
    ($to:ty, $kind:path, $name:expr) => {
        impl TryFrom<AnyConnectOptions> for $to {
            type Error = Error;

            fn try_from(value: AnyConnectOptions) -> Result<Self, Self::Error> {
                #[allow(irrefutable_let_patterns)]
                if let $kind(connect_options) = value.0 {
                    Ok(connect_options)
                } else {
                    Err(Error::Configuration(
                        format!("Not {} typed AnyConnectOptions", $name).into(),
                    ))
                }
            }
        }

        impl AnyConnectOptions {
            paste::item! {
                pub fn [< as_ $name >] (&self) -> Option<&$to> {
                    #[allow(irrefutable_let_patterns)]
                    if let $kind(ref connect_options) = self.0 {
                        Some(connect_options)
                    } else {
                        None
                    }
                }

                pub fn [< as_ $name _mut >] (&mut self) -> Option<&mut $to> {
                    #[allow(irrefutable_let_patterns)]
                    if let $kind(ref mut connect_options) = self.0 {
                        Some(connect_options)
                    } else {
                        None
                    }
                }
            }
        }
    };
}

#[cfg(feature = "postgres")]
try_from_any_connect_options_to!(
    PgConnectOptions,
    AnyConnectOptionsKind::Postgres,
    "postgres"
);

#[cfg(feature = "mysql")]
try_from_any_connect_options_to!(MySqlConnectOptions, AnyConnectOptionsKind::MySql, "mysql");

#[cfg(feature = "sqlite")]
try_from_any_connect_options_to!(
    SqliteConnectOptions,
    AnyConnectOptionsKind::Sqlite,
    "sqlite"
);

#[cfg(feature = "mssql")]
try_from_any_connect_options_to!(MssqlConnectOptions, AnyConnectOptionsKind::Mssql, "mssql");

#[derive(Debug, Clone)]
pub(crate) enum AnyConnectOptionsKind {
    #[cfg(feature = "postgres")]
    Postgres(PgConnectOptions),

    #[cfg(feature = "mysql")]
    MySql(MySqlConnectOptions),

    #[cfg(feature = "sqlite")]
    Sqlite(SqliteConnectOptions),

    #[cfg(feature = "mssql")]
    Mssql(MssqlConnectOptions),
}

#[cfg(feature = "postgres")]
impl From<PgConnectOptions> for AnyConnectOptions {
    fn from(options: PgConnectOptions) -> Self {
        Self(AnyConnectOptionsKind::Postgres(options))
    }
}

#[cfg(feature = "mysql")]
impl From<MySqlConnectOptions> for AnyConnectOptions {
    fn from(options: MySqlConnectOptions) -> Self {
        Self(AnyConnectOptionsKind::MySql(options))
    }
}

#[cfg(feature = "sqlite")]
impl From<SqliteConnectOptions> for AnyConnectOptions {
    fn from(options: SqliteConnectOptions) -> Self {
        Self(AnyConnectOptionsKind::Sqlite(options))
    }
}

#[cfg(feature = "mssql")]
impl From<MssqlConnectOptions> for AnyConnectOptions {
    fn from(options: MssqlConnectOptions) -> Self {
        Self(AnyConnectOptionsKind::Mssql(options))
    }
}

impl FromStr for AnyConnectOptions {
    type Err = Error;

    fn from_str(url: &str) -> Result<Self, Self::Err> {
        match AnyKind::from_str(url)? {
            #[cfg(feature = "postgres")]
            AnyKind::Postgres => {
                PgConnectOptions::from_str(url).map(AnyConnectOptionsKind::Postgres)
            }

            #[cfg(feature = "mysql")]
            AnyKind::MySql => MySqlConnectOptions::from_str(url).map(AnyConnectOptionsKind::MySql),

            #[cfg(feature = "sqlite")]
            AnyKind::Sqlite => {
                SqliteConnectOptions::from_str(url).map(AnyConnectOptionsKind::Sqlite)
            }

            #[cfg(feature = "mssql")]
            AnyKind::Mssql => MssqlConnectOptions::from_str(url).map(AnyConnectOptionsKind::Mssql),
        }
        .map(AnyConnectOptions)
    }
}

impl ConnectOptions for AnyConnectOptions {
    type Connection = AnyConnection;

    #[inline]
    fn connect(&self) -> BoxFuture<'_, Result<AnyConnection, Error>> {
        Box::pin(AnyConnection::establish(self))
    }

    fn log_statements(&mut self, level: LevelFilter) -> &mut Self {
        match &mut self.0 {
            #[cfg(feature = "postgres")]
            AnyConnectOptionsKind::Postgres(o) => {
                o.log_statements(level);
            }

            #[cfg(feature = "mysql")]
            AnyConnectOptionsKind::MySql(o) => {
                o.log_statements(level);
            }

            #[cfg(feature = "sqlite")]
            AnyConnectOptionsKind::Sqlite(o) => {
                o.log_statements(level);
            }

            #[cfg(feature = "mssql")]
            AnyConnectOptionsKind::Mssql(o) => {
                o.log_statements(level);
            }
        };
        self
    }

    fn log_slow_statements(&mut self, level: LevelFilter, duration: Duration) -> &mut Self {
        match &mut self.0 {
            #[cfg(feature = "postgres")]
            AnyConnectOptionsKind::Postgres(o) => {
                o.log_slow_statements(level, duration);
            }

            #[cfg(feature = "mysql")]
            AnyConnectOptionsKind::MySql(o) => {
                o.log_slow_statements(level, duration);
            }

            #[cfg(feature = "sqlite")]
            AnyConnectOptionsKind::Sqlite(o) => {
                o.log_slow_statements(level, duration);
            }

            #[cfg(feature = "mssql")]
            AnyConnectOptionsKind::Mssql(o) => {
                o.log_slow_statements(level, duration);
            }
        };
        self
    }
}
