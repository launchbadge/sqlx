use crate::any::{Any, AnyConnection};
use crate::error::Error;
use crate::transaction::TransactionManager;

pub struct AnyTransactionManager;

impl TransactionManager for AnyTransactionManager {
    type Database = Any;

    async fn begin(conn: &mut AnyConnection) -> Result<(), Error> {
        conn.backend.begin().await
    }

    async fn commit(conn: &mut AnyConnection) -> Result<(), Error> {
        conn.backend.commit().await
    }

    async fn rollback(conn: &mut AnyConnection) -> Result<(), Error> {
        conn.backend.rollback().await
    }

    fn start_rollback(conn: &mut AnyConnection) {
        conn.backend.start_rollback()
    }
}
