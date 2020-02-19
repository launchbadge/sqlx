use std::future::Future;
use std::pin::Pin;
use std::task::{Context, Poll};

use futures_core::future::BoxFuture;
use futures_core::stream::BoxStream;

use crate::cursor::Cursor;
use crate::database::HasRow;
use crate::postgres::protocol::StatementId;
use crate::postgres::PgConnection;
use crate::Postgres;

pub struct PgCursor<'a> {
    statement: StatementId,
    connection: &'a mut PgConnection,
}

impl<'a> PgCursor<'a> {
    pub(super) fn from_connection(
        connection: &'a mut PgConnection,
        statement: StatementId,
    ) -> Self {
        Self {
            connection,
            statement,
        }
    }
}

impl<'a> Cursor<'a> for PgCursor<'a> {
    type Database = Postgres;

    fn first(self) -> BoxFuture<'a, crate::Result<Option<<Self::Database as HasRow>::Row>>> {
        todo!()
    }

    fn next(&mut self) -> BoxFuture<crate::Result<Option<<Self::Database as HasRow>::Row>>> {
        todo!()
    }

    fn map<T, F>(self, f: F) -> BoxStream<'a, crate::Result<T>>
    where
        F: Fn(<Self::Database as HasRow>::Row) -> T,
    {
        todo!()
    }
}

impl<'a> Future for PgCursor<'a> {
    type Output = crate::Result<u64>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        todo!()
    }
}
