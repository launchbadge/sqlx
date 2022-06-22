use futures_util::future::BoxFuture;

use crate::any::connection::AnyConnectionKind;
use crate::any::{Any, AnyConnection};
use crate::database::Database;
use crate::error::Error;
#[cfg(feature = "mysql")]
use crate::mysql::MySqlTransactionOptions;
#[cfg(feature = "postgres")]
use crate::postgres::PgTransactionOptions;
#[cfg(feature = "sqlite")]
use crate::sqlite::SqliteTransactionOptions;
use crate::transaction::TransactionManager;

/// Transaction manager for generic database connection.
pub struct AnyTransactionManager;

impl TransactionManager for AnyTransactionManager {
    type Database = Any;
    type Options = AnyTransactionOptions;

    fn begin_with(
        conn: &mut AnyConnection,
        options: AnyTransactionOptions,
    ) -> BoxFuture<'_, Result<(), Error>> {
        match &mut conn.0 {
            #[cfg(feature = "postgres")]
            AnyConnectionKind::Postgres(conn) => {
                <crate::postgres::Postgres as Database>::TransactionManager::begin_with(
                    conn,
                    options.postgres,
                )
            }

            #[cfg(feature = "mysql")]
            AnyConnectionKind::MySql(conn) => {
                <crate::mysql::MySql as Database>::TransactionManager::begin_with(
                    conn,
                    options.mysql,
                )
            }

            #[cfg(feature = "sqlite")]
            AnyConnectionKind::Sqlite(conn) => {
                <crate::sqlite::Sqlite as Database>::TransactionManager::begin_with(
                    conn,
                    options.sqlite,
                )
            }

            #[cfg(feature = "mssql")]
            AnyConnectionKind::Mssql(conn) => {
                <crate::mssql::Mssql as Database>::TransactionManager::begin_with(
                    conn,
                    Default::default(),
                )
            }
        }
    }

    fn commit(conn: &mut AnyConnection) -> BoxFuture<'_, Result<(), Error>> {
        match &mut conn.0 {
            #[cfg(feature = "postgres")]
            AnyConnectionKind::Postgres(conn) => {
                <crate::postgres::Postgres as Database>::TransactionManager::commit(conn)
            }

            #[cfg(feature = "mysql")]
            AnyConnectionKind::MySql(conn) => {
                <crate::mysql::MySql as Database>::TransactionManager::commit(conn)
            }

            #[cfg(feature = "sqlite")]
            AnyConnectionKind::Sqlite(conn) => {
                <crate::sqlite::Sqlite as Database>::TransactionManager::commit(conn)
            }

            #[cfg(feature = "mssql")]
            AnyConnectionKind::Mssql(conn) => {
                <crate::mssql::Mssql as Database>::TransactionManager::commit(conn)
            }
        }
    }

    fn rollback(conn: &mut AnyConnection) -> BoxFuture<'_, Result<(), Error>> {
        match &mut conn.0 {
            #[cfg(feature = "postgres")]
            AnyConnectionKind::Postgres(conn) => {
                <crate::postgres::Postgres as Database>::TransactionManager::rollback(conn)
            }

            #[cfg(feature = "mysql")]
            AnyConnectionKind::MySql(conn) => {
                <crate::mysql::MySql as Database>::TransactionManager::rollback(conn)
            }

            #[cfg(feature = "sqlite")]
            AnyConnectionKind::Sqlite(conn) => {
                <crate::sqlite::Sqlite as Database>::TransactionManager::rollback(conn)
            }

            #[cfg(feature = "mssql")]
            AnyConnectionKind::Mssql(conn) => {
                <crate::mssql::Mssql as Database>::TransactionManager::rollback(conn)
            }
        }
    }

    fn start_rollback(conn: &mut AnyConnection) {
        match &mut conn.0 {
            #[cfg(feature = "postgres")]
            AnyConnectionKind::Postgres(conn) => {
                <crate::postgres::Postgres as Database>::TransactionManager::start_rollback(conn)
            }

            #[cfg(feature = "mysql")]
            AnyConnectionKind::MySql(conn) => {
                <crate::mysql::MySql as Database>::TransactionManager::start_rollback(conn)
            }

            #[cfg(feature = "sqlite")]
            AnyConnectionKind::Sqlite(conn) => {
                <crate::sqlite::Sqlite as Database>::TransactionManager::start_rollback(conn)
            }

            #[cfg(feature = "mssql")]
            AnyConnectionKind::Mssql(conn) => {
                <crate::mssql::Mssql as Database>::TransactionManager::start_rollback(conn)
            }
        }
    }
}

/// Transaction initiation options for generic database connection.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct AnyTransactionOptions {
    /// Options that are used when the connection is to SQLite.
    #[cfg(feature = "sqlite")]
    pub(crate) sqlite: SqliteTransactionOptions,

    /// Options that are used when the connection is to Postgres.
    #[cfg(feature = "postgres")]
    pub(crate) postgres: PgTransactionOptions,

    /// Options that are used when the connection is to MySQL.
    #[cfg(feature = "mysql")]
    pub(crate) mysql: MySqlTransactionOptions,
}

impl AnyTransactionOptions {
    #[cfg(feature = "postgres")]
    pub fn postgres(self, postgres: PgTransactionOptions) -> Self {
        Self { postgres, ..self }
    }

    #[cfg(feature = "sqlite")]
    pub fn sqlite(self, sqlite: SqliteTransactionOptions) -> Self {
        Self { sqlite, ..self }
    }

    #[cfg(feature = "mysql")]
    pub fn mysql(self, mysql: MySqlTransactionOptions) -> Self {
        Self { mysql, ..self }
    }
}
