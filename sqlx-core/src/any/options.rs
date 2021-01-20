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
/// a connection uri.
///
/// ```text
/// postgres://postgres:password@localhost/database
/// mysql://root:password@localhost/database
/// ```
#[derive(Debug)]
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

#[derive(Debug)]
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

// Conceptually:
// fn map<F>(self, f: F)
// where
//     F: for<T: ConnectOptions> FnOnce(T) -> T
macro_rules! map {
    ($this:ident, $f:expr) => {
        Self(match $this.0 {
            #[cfg(feature = "postgres")]
            AnyConnectOptionsKind::Postgres(opt) => {
                let map_fn = map_fn_type_hint::<PgConnectOptions, _>($f);
                AnyConnectOptionsKind::Postgres(map_fn(opt))
            }

            #[cfg(feature = "mysql")]
            AnyConnectOptionsKind::MySql(opt) => {
                let map_fn = map_fn_type_hint::<MySqlConnectOptions, _>($f);
                AnyConnectOptionsKind::MySql(map_fn(opt))
            }

            #[cfg(feature = "sqlite")]
            AnyConnectOptionsKind::Sqlite(opt) => {
                let map_fn = map_fn_type_hint::<SqliteConnectOptions, _>($f);
                AnyConnectOptionsKind::Sqlite(map_fn(opt))
            }

            #[cfg(feature = "mssql")]
            AnyConnectOptionsKind::Mssql(opt) => {
                let map_fn = map_fn_type_hint::<MssqlConnectOptions, _>($f);
                AnyConnectOptionsKind::Mssql(map_fn(opt))
            }
        })
    };
}

fn map_fn_type_hint<T, F>(f: F) -> impl FnOnce(T) -> T
where
    F: FnOnce(T) -> T,
{
    f
}

impl ConnectOptions for AnyConnectOptions {
    type Connection = AnyConnection;

    #[inline]
    fn connect(&self) -> BoxFuture<'_, Result<AnyConnection, Error>> {
        Box::pin(AnyConnection::establish(self))
    }

    fn log_statements(self, level: LevelFilter) -> Self {
        map!(self, move |o| o.log_statements(level))
    }

    fn log_slow_statements(self, level: LevelFilter, duration: Duration) -> Self {
        map!(self, move |o| o.log_slow_statements(level, duration))
    }
}
