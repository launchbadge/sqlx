use futures_util::future::BoxFuture;

use crate::any::{Any, AnyConnection};
use crate::error::Error;
use crate::transaction::TransactionManager;

pub struct AnyTransactionManager;

impl TransactionManager for AnyTransactionManager {
    type Database = Any;

    fn begin(conn: &mut AnyConnection) -> BoxFuture<'_, Result<(), Error>> {
        conn.backend.begin()
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
