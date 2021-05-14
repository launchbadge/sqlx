use crate::error::Error;
use std::str::FromStr;

#[derive(Debug)]
pub enum AnyKind {
    #[cfg(feature = "postgres")]
    Postgres,

    #[cfg(feature = "mysql")]
    MySql,

    #[cfg(feature = "sqlite")]
    Sqlite,

    #[cfg(feature = "mssql")]
    Mssql,
}

impl FromStr for AnyKind {
    type Err = Error;

    fn from_str(uri: &str) -> Result<Self, Self::Err> {
        match uri {
            #[cfg(feature = "postgres")]
            _ if uri.starts_with("postgres:") || uri.starts_with("postgresql:") => {
                Ok(AnyKind::Postgres)
            }

            #[cfg(not(feature = "postgres"))]
            _ if uri.starts_with("postgres:") || uri.starts_with("postgresql:") => {
                Err(Error::Configuration("database URL has the scheme of a PostgreSQL database but the `postgres` feature is not enabled".into()))
            }

            #[cfg(feature = "mysql")]
            _ if uri.starts_with("mysql:") || uri.starts_with("mariadb:") => {
                Ok(AnyKind::MySql)
            }

            #[cfg(not(feature = "mysql"))]
            _ if uri.starts_with("mysql:") || uri.starts_with("mariadb:") => {
                Err(Error::Configuration("database URL has the scheme of a MySQL database but the `mysql` feature is not enabled".into()))
            }

            #[cfg(feature = "sqlite")]
            _ if uri.starts_with("sqlite:") => {
                Ok(AnyKind::Sqlite)
            }

            #[cfg(not(feature = "sqlite"))]
            _ if uri.starts_with("sqlite:") => {
                Err(Error::Configuration("database URL has the scheme of a SQLite database but the `sqlite` feature is not enabled".into()))
            }

            #[cfg(feature = "mssql")]
            _ if uri.starts_with("mssql:") || uri.starts_with("sqlserver:") => {
                Ok(AnyKind::Mssql)
            }

            #[cfg(not(feature = "mssql"))]
            _ if uri.starts_with("mssql:") || uri.starts_with("sqlserver:") => {
                Err(Error::Configuration("database URL has the scheme of a MSSQL database but the `mssql` feature is not enabled".into()))
            }

            _ => Err(Error::Configuration(format!("unrecognized database url: {:?}", uri).into()))
        }
    }
}

pub trait AnyQuery<'q> : Send + Sized + std::marker::Sync {
    fn build(&self, kind: Option<AnyKind>) -> &'q str;
}

impl<'q> AnyQuery<'q> for &'q str {
    fn build(&self, _kind: Option<AnyKind>) -> &'q str {
        self
    }
}
