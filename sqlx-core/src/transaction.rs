use std::ops::{Deref, DerefMut};

use async_std::task;
use futures_core::future::BoxFuture;
use futures_core::stream::BoxStream;

use crate::connection::Connection;
use crate::database::Database;
use crate::describe::Describe;
use crate::executor::Executor;

pub struct Transaction<T>
where
    T: Connection + Send + 'static,
{
    inner: Option<T>,
    depth: u32,
}

impl<T> Transaction<T>
where
    T: Connection + Send + 'static,
{
    pub(crate) async fn new(depth: u32, mut inner: T) -> crate::Result<Self> {
        if depth == 0 {
            inner.send("BEGIN").await?;
        } else {
            inner
                .send(&format!("SAVEPOINT _sqlx_savepoint_{}", depth))
                .await?;
        }

        Ok(Self {
            inner: Some(inner),
            depth: depth + 1,
        })
    }

    pub async fn begin(mut self) -> crate::Result<Transaction<T>> {
        Transaction::new(self.depth, self.inner.take().expect(ERR_FINALIZED)).await
    }

    pub async fn commit(mut self) -> crate::Result<T> {
        let mut inner = self.inner.take().expect(ERR_FINALIZED);
        let depth = self.depth;

        if depth == 1 {
            inner.send("COMMIT").await?;
        } else {
            inner
                .send(&format!("RELEASE SAVEPOINT _sqlx_savepoint_{}", depth - 1))
                .await?;
        }

        Ok(inner)
    }

    pub async fn rollback(mut self) -> crate::Result<T> {
        let mut inner = self.inner.take().expect(ERR_FINALIZED);
        let depth = self.depth;

        if depth == 1 {
            inner.send("ROLLBACK").await?;
        } else {
            inner
                .send(&format!(
                    "ROLLBACK TO SAVEPOINT _sqlx_savepoint_{}",
                    depth - 1
                ))
                .await?;
        }

        Ok(inner)
    }
}

const ERR_FINALIZED: &str = "(bug) transaction already finalized";

impl<Conn> Deref for Transaction<Conn>
where
    Conn: Connection,
{
    type Target = Conn;

    fn deref(&self) -> &Self::Target {
        self.inner.as_ref().expect(ERR_FINALIZED)
    }
}

impl<Conn> DerefMut for Transaction<Conn>
where
    Conn: Connection,
{
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.inner.as_mut().expect(ERR_FINALIZED)
    }
}

impl<T> Connection for Transaction<T>
where
    T: Connection,
{
    // Close is equivalent to ROLLBACK followed by CLOSE
    fn close(self) -> BoxFuture<'static, crate::Result<()>> {
        Box::pin(async move { self.rollback().await?.close().await })
    }
}

impl<T> Executor for Transaction<T>
where
    T: Connection,
{
    type Database = T::Database;

    fn send<'e, 'q: 'e>(&'e mut self, commands: &'q str) -> BoxFuture<'e, crate::Result<()>> {
        self.deref_mut().send(commands)
    }

    fn execute<'e, 'q: 'e>(
        &'e mut self,
        query: &'q str,
        args: <Self::Database as Database>::Arguments,
    ) -> BoxFuture<'e, crate::Result<u64>> {
        self.deref_mut().execute(query, args)
    }

    fn fetch<'e, 'q: 'e>(
        &'e mut self,
        query: &'q str,
        args: <Self::Database as Database>::Arguments,
    ) -> BoxStream<'e, crate::Result<<Self::Database as Database>::Row>> {
        self.deref_mut().fetch(query, args)
    }

    fn fetch_optional<'e, 'q: 'e>(
        &'e mut self,
        query: &'q str,
        args: <Self::Database as Database>::Arguments,
    ) -> BoxFuture<'e, crate::Result<Option<<Self::Database as Database>::Row>>> {
        self.deref_mut().fetch_optional(query, args)
    }

    fn describe<'e, 'q: 'e>(
        &'e mut self,
        query: &'q str,
    ) -> BoxFuture<'e, crate::Result<Describe<Self::Database>>> {
        self.deref_mut().describe(query)
    }
}

impl<Conn> Drop for Transaction<Conn>
where
    Conn: Connection,
{
    fn drop(&mut self) {
        if self.depth > 0 {
            if let Some(mut inner) = self.inner.take() {
                task::spawn(async move {
                    let res = inner.send("ROLLBACK").await;

                    // If the rollback failed we need to close the inner connection
                    if res.is_err() {
                        // This will explicitly forget the connection so it will not
                        // return to the pool
                        let _ = inner.close().await;
                    }
                });
            }
        }
    }
}
