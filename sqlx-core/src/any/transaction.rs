use std::borrow::Cow;
use std::future::Future;

use crate::any::{Any, AnyConnection};
use crate::database::Database;
use crate::error::Error;
use crate::transaction::TransactionManager;

pub struct AnyTransactionManager;

impl TransactionManager for AnyTransactionManager {
    type Database = Any;

    fn begin<'conn>(
        conn: &'conn mut AnyConnection,
        statement: Option<Cow<'static, str>>,
    ) -> impl Future<Output = Result<(), Error>> + Send + 'conn {
        conn.backend.begin(statement)
    }

    fn commit(conn: &mut AnyConnection) -> impl Future<Output = Result<(), Error>> + Send + '_ {
        conn.backend.commit()
    }

    fn rollback(conn: &mut AnyConnection) -> impl Future<Output = Result<(), Error>> + Send + '_ {
        conn.backend.rollback()
    }

    fn start_rollback(conn: &mut AnyConnection) {
        conn.backend.start_rollback()
    }

    fn get_transaction_depth(conn: &<Self::Database as Database>::Connection) -> usize {
        conn.backend.get_transaction_depth()
    }
}
