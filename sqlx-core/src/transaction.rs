use std::ops::{Deref, DerefMut};

use futures_core::future::BoxFuture;

use crate::connection::Connection;
use crate::database::HasCursor;
use crate::executor::{Execute, Executor};
use crate::runtime::spawn;
use crate::Database;

// Transaction<PoolConnection<PgConnection>>
// Transaction<PgConnection>
pub struct Transaction<T>
where
    T: Connection,
    T: Executor<'static>,
{
    inner: Option<T>,
    depth: u32,
}

impl<T> Transaction<T>
where
    T: Connection,
    T: Executor<'static>,
{
    pub(crate) async fn new(depth: u32, mut inner: T) -> crate::Result<Self> {
        if depth == 0 {
            inner.execute_by_ref("BEGIN").await?;
        } else {
            let stmt = format!("SAVEPOINT _sqlx_savepoint_{}", depth);

            inner.execute_by_ref(&*stmt).await?;
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
            inner.execute_by_ref("COMMIT").await?;
        } else {
            let stmt = format!("RELEASE SAVEPOINT _sqlx_savepoint_{}", depth - 1);

            inner.execute_by_ref(&*stmt).await?;
        }

        Ok(inner)
    }

    pub async fn rollback(mut self) -> crate::Result<T> {
        let mut inner = self.inner.take().expect(ERR_FINALIZED);
        let depth = self.depth;

        if depth == 1 {
            inner.execute_by_ref("ROLLBACK").await?;
        } else {
            let stmt = format!("ROLLBACK TO SAVEPOINT _sqlx_savepoint_{}", depth - 1);

            inner.execute_by_ref(&*stmt).await?;
        }

        Ok(inner)
    }
}

const ERR_FINALIZED: &str = "(bug) transaction already finalized";

impl<T> Deref for Transaction<T>
where
    T: Connection,
    T: Executor<'static>,
{
    type Target = T;

    fn deref(&self) -> &Self::Target {
        self.inner.as_ref().expect(ERR_FINALIZED)
    }
}

impl<T> DerefMut for Transaction<T>
where
    T: Connection,
    T: Executor<'static>,
{
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.inner.as_mut().expect(ERR_FINALIZED)
    }
}

impl<T> Connection for Transaction<T>
where
    T: Connection,
    T: Executor<'static>,
{
    type Database = <T as Connection>::Database;

    // Close is equivalent to ROLLBACK followed by CLOSE
    fn close(self) -> BoxFuture<'static, crate::Result<()>> {
        Box::pin(async move { self.rollback().await?.close().await })
    }
}

impl<'a, DB, T> Executor<'a> for &'a mut Transaction<T>
where
    DB: Database,
    T: Connection<Database = DB>,
    T: Executor<'static, Database = DB>,
{
    type Database = <T as Connection>::Database;

    fn execute<'b, E>(self, query: E) -> <<T as Connection>::Database as HasCursor<'a>>::Cursor
    where
        E: Execute<'b, Self::Database>,
    {
        (**self).execute_by_ref(query)
    }

    fn execute_by_ref<'b, 'c, E>(
        &'c mut self,
        query: E,
    ) -> <Self::Database as HasCursor<'c>>::Cursor
    where
        E: Execute<'b, Self::Database>,
    {
        (**self).execute_by_ref(query)
    }
}

impl<T> Drop for Transaction<T>
where
    T: Connection,
    T: Executor<'static>,
{
    fn drop(&mut self) {
        if self.depth > 0 {
            if let Some(mut inner) = self.inner.take() {
                spawn(async move {
                    let res = inner.execute_by_ref("ROLLBACK").await;

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
