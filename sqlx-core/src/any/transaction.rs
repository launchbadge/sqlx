use futures_util::future::BoxFuture;
use std::borrow::Cow;

use crate::any::{Any, AnyConnection};
use crate::database::Database;
use crate::error::Error;
use crate::transaction::TransactionManager;

pub struct AnyTransactionManager;

impl TransactionManager for AnyTransactionManager {
    type Database = Any;

    fn begin(conn: &mut AnyConnection) -> BoxFuture<'_, Result<(), Error>> {
        conn.backend.begin()
    }

    fn begin_with<'a, S>(
        conn: &'a mut <Self::Database as Database>::Connection,
        sql: S,
    ) -> BoxFuture<'a, Result<(), Error>>
    where
        S: Into<Cow<'static, str>> + Send + 'a,
    {
        conn.backend.begin_with(sql.into())
    }

    fn commit(conn: &mut AnyConnection) -> BoxFuture<'_, Result<(), Error>> {
        conn.backend.commit()
    }

    fn rollback(conn: &mut AnyConnection) -> BoxFuture<'_, Result<(), Error>> {
        conn.backend.rollback()
    }

    fn start_rollback(conn: &mut AnyConnection) {
        conn.backend.start_rollback()
    }
}
