use futures_util::future::BoxFuture;

use crate::any::connection::AnyConnectionKind;
use crate::any::{Any, AnyConnection};
use crate::database::Database;
use crate::error::Error;
use crate::transaction::TransactionManager;

pub struct AnyTransactionManager;

impl TransactionManager for AnyTransactionManager {
    type Database = Any;

    fn begin(conn: &mut AnyConnection, depth: usize) -> BoxFuture<'_, Result<(), Error>> {
        match &mut conn.0 {
            #[cfg(feature = "postgres")]
            AnyConnectionKind::Postgres(conn) => {
                <crate::postgres::Postgres as Database>::TransactionManager::begin(conn, depth)
            }

            #[cfg(feature = "mysql")]
            AnyConnectionKind::MySql(conn) => {
                <crate::mysql::MySql as Database>::TransactionManager::begin(conn, depth)
            }

            #[cfg(feature = "sqlite")]
            AnyConnectionKind::Sqlite(conn) => {
                <crate::sqlite::Sqlite as Database>::TransactionManager::begin(conn, depth)
            }

            #[cfg(feature = "mssql")]
            AnyConnectionKind::Mssql(conn) => {
                <crate::mssql::Mssql as Database>::TransactionManager::begin(conn, depth)
            }
        }
    }

    fn commit(conn: &mut AnyConnection, depth: usize) -> BoxFuture<'_, Result<(), Error>> {
        match &mut conn.0 {
            #[cfg(feature = "postgres")]
            AnyConnectionKind::Postgres(conn) => {
                <crate::postgres::Postgres as Database>::TransactionManager::commit(conn, depth)
            }

            #[cfg(feature = "mysql")]
            AnyConnectionKind::MySql(conn) => {
                <crate::mysql::MySql as Database>::TransactionManager::commit(conn, depth)
            }

            #[cfg(feature = "sqlite")]
            AnyConnectionKind::Sqlite(conn) => {
                <crate::sqlite::Sqlite as Database>::TransactionManager::commit(conn, depth)
            }

            #[cfg(feature = "mssql")]
            AnyConnectionKind::Mssql(conn) => {
                <crate::mssql::Mssql as Database>::TransactionManager::commit(conn, depth)
            }
        }
    }

    fn rollback(conn: &mut AnyConnection, depth: usize) -> BoxFuture<'_, Result<(), Error>> {
        match &mut conn.0 {
            #[cfg(feature = "postgres")]
            AnyConnectionKind::Postgres(conn) => {
                <crate::postgres::Postgres as Database>::TransactionManager::rollback(conn, depth)
            }

            #[cfg(feature = "mysql")]
            AnyConnectionKind::MySql(conn) => {
                <crate::mysql::MySql as Database>::TransactionManager::rollback(conn, depth)
            }

            #[cfg(feature = "sqlite")]
            AnyConnectionKind::Sqlite(conn) => {
                <crate::sqlite::Sqlite as Database>::TransactionManager::rollback(conn, depth)
            }

            #[cfg(feature = "mssql")]
            AnyConnectionKind::Mssql(conn) => {
                <crate::mssql::Mssql as Database>::TransactionManager::rollback(conn, depth)
            }
        }
    }

    fn start_rollback(conn: &mut AnyConnection, depth: usize) {
        match &mut conn.0 {
            #[cfg(feature = "postgres")]
            AnyConnectionKind::Postgres(conn) => {
                <crate::postgres::Postgres as Database>::TransactionManager::start_rollback(
                    conn, depth,
                )
            }

            #[cfg(feature = "mysql")]
            AnyConnectionKind::MySql(conn) => {
                <crate::mysql::MySql as Database>::TransactionManager::start_rollback(conn, depth)
            }

            #[cfg(feature = "sqlite")]
            AnyConnectionKind::Sqlite(conn) => {
                <crate::sqlite::Sqlite as Database>::TransactionManager::start_rollback(conn, depth)
            }

            #[cfg(feature = "mssql")]
            AnyConnectionKind::Mssql(conn) => {
                <crate::mssql::Mssql as Database>::TransactionManager::start_rollback(conn, depth)
            }
        }
    }
}
