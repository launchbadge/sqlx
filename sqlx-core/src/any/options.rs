use crate::any::AnyConnection;
use crate::connection::ConnectOptions;
use crate::error::Error;
use futures_core::future::BoxFuture;
use std::str::FromStr;

#[cfg(feature = "postgres")]
use crate::postgres::PgConnectOptions;

#[cfg(feature = "mysql")]
use crate::mysql::MySqlConnectOptions;

#[cfg(feature = "sqlite")]
use crate::sqlite::SqliteConnectOptions;

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

impl FromStr for AnyConnectOptions {
    type Err = Error;

    fn from_str(url: &str) -> Result<Self, Self::Err> {
        match url {
            #[cfg(feature = "postgres")]
            _ if url.starts_with("postgres:") || url.starts_with("postgresql:") => {
                PgConnectOptions::from_str(url).map(AnyConnectOptionsKind::Postgres)
            }

            #[cfg(not(feature = "postgres"))]
            _ if url.starts_with("postgres:") || url.starts_with("postgresql:") => {
                Err("database URL has the scheme of a PostgreSQL database but the `postgres` feature is not enabled".into())
            }

            #[cfg(feature = "mysql")]
            _ if url.starts_with("mysql:") || url.starts_with("mariadb:") => {
                MySqlConnectOptions::from_str(url).map(AnyConnectOptionsKind::MySql)
            }

            #[cfg(not(feature = "mysql"))]
            _ if url.starts_with("mysql:") || url.starts_with("mariadb:") => {
                Err("database URL has the scheme of a MySQL database but the `mysql` feature is not enabled".into())
            }

            #[cfg(feature = "sqlite")]
            _ if url.starts_with("sqlite:") => {
                SqliteConnectOptions::from_str(url).map(AnyConnectOptionsKind::Sqlite)
            }

            #[cfg(not(feature = "sqlite"))]
            _ if url.starts_with("sqlite:") => {
                Err("database URL has the scheme of a SQLite database but the `sqlite` feature is not enabled".into())
            }

            #[cfg(feature = "mssql")]
            _ if url.starts_with("mssql:") || url.starts_with("sqlserver:") => {
                MssqlConnectOptions::from_str(url).map(AnyConnectOptionsKind::Mssql)
            }

            #[cfg(not(feature = "mssql"))]
            _ if url.starts_with("mssql:") || url.starts_with("sqlserver:") => {
                Err("database URL has the scheme of a MSSQL database but the `mssql` feature is not enabled".into())
            }

            _ => Err(Error::Configuration(format!("unrecognized database url: {:?}", url).into()))
        }.map(AnyConnectOptions)
    }
}

impl ConnectOptions for AnyConnectOptions {
    type Connection = AnyConnection;

    #[inline]
    fn connect(&self) -> BoxFuture<'_, Result<AnyConnection, Error>> {
        Box::pin(AnyConnection::establish(self))
    }
}
