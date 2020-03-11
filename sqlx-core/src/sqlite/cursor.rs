use futures_core::future::BoxFuture;

use crate::connection::MaybeOwnedConnection;
use crate::cursor::Cursor;
use crate::executor::Execute;
use crate::pool::Pool;
use crate::sqlite::connection::SqliteConnection;
use crate::sqlite::{Sqlite, SqliteRow};

pub struct SqliteCursor<'c, 'q> {
    c: std::marker::PhantomData<&'c ()>,
    q: std::marker::PhantomData<&'q ()>,
}

impl<'c, 'q> Cursor<'c, 'q> for SqliteCursor<'c, 'q> {
    type Database = Sqlite;

    #[doc(hidden)]
    fn from_pool<E>(pool: &Pool<SqliteConnection>, query: E) -> Self
    where
        Self: Sized,
        E: Execute<'q, Sqlite>,
    {
        todo!()
    }

    #[doc(hidden)]
    fn from_connection<E, C>(conn: C, query: E) -> Self
    where
        Self: Sized,
        C: Into<MaybeOwnedConnection<'c, SqliteConnection>>,
        E: Execute<'q, Sqlite>,
    {
        todo!()
    }

    fn next(&mut self) -> BoxFuture<crate::Result<Option<SqliteRow<'_>>>> {
        todo!()
    }
}
