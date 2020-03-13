use std::ops::{Deref, DerefMut};

use futures_core::future::BoxFuture;

use crate::connection::Connection;
use crate::database::Database;
use crate::database::HasCursor;
use crate::describe::Describe;
use crate::executor::{Execute, Executor, RefExecutor};
use crate::runtime::spawn;

// Transaction<PoolConnection<PgConnection>>
// Transaction<PgConnection>
pub struct Transaction<T>
where
    T: Connection,
{
    inner: Option<T>,
    depth: u32,
}

impl<T> Transaction<T>
where
    T: Connection,
{
    pub(crate) async fn new(depth: u32, mut inner: T) -> crate::Result<Self> {
        if depth == 0 {
            inner.execute("BEGIN").await?;
        } else {
            let stmt = format!("SAVEPOINT _sqlx_savepoint_{}", depth);

            inner.execute(&*stmt).await?;
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
            inner.execute("COMMIT").await?;
        } else {
            let stmt = format!("RELEASE SAVEPOINT _sqlx_savepoint_{}", depth - 1);

            inner.execute(&*stmt).await?;
        }

        Ok(inner)
    }

    pub async fn rollback(mut self) -> crate::Result<T> {
        let mut inner = self.inner.take().expect(ERR_FINALIZED);
        let depth = self.depth;

        if depth == 1 {
            inner.execute("ROLLBACK").await?;
        } else {
            let stmt = format!("ROLLBACK TO SAVEPOINT _sqlx_savepoint_{}", depth - 1);

            inner.execute(&*stmt).await?;
        }

        Ok(inner)
    }
}

const ERR_FINALIZED: &str = "(bug) transaction already finalized";

impl<T> Deref for Transaction<T>
where
    T: Connection,
{
    type Target = T;

    fn deref(&self) -> &Self::Target {
        self.inner.as_ref().expect(ERR_FINALIZED)
    }
}

impl<T> DerefMut for Transaction<T>
where
    T: Connection,
{
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.inner.as_mut().expect(ERR_FINALIZED)
    }
}

impl<'c, DB, T> Executor for &'c mut Transaction<T>
where
    DB: Database,
    T: Connection<Database = DB>,
{
    type Database = T::Database;

    fn execute<'e, 'q, E: 'e>(&'e mut self, query: E) -> BoxFuture<'e, crate::Result<u64>>
    where
        E: Execute<'q, Self::Database>,
    {
        (**self).execute(query)
    }

    fn fetch<'q, 'e, E>(&'e mut self, query: E) -> <Self::Database as HasCursor<'e, 'q>>::Cursor
    where
        E: Execute<'q, Self::Database>,
    {
        (**self).fetch(query)
    }

    fn describe<'e, 'q, E: 'e>(
        &'e mut self,
        query: E,
    ) -> BoxFuture<'e, crate::Result<Describe<Self::Database>>>
    where
        E: Execute<'q, Self::Database>,
    {
        (**self).describe(query)
    }
}

impl<'e, DB, T> RefExecutor<'e> for &'e mut Transaction<T>
where
    DB: Database,
    T: Connection<Database = DB>,
{
    type Database = DB;

    fn fetch_by_ref<'q, E>(self, query: E) -> <Self::Database as HasCursor<'e, 'q>>::Cursor
    where
        E: Execute<'q, Self::Database>,
    {
        (**self).fetch(query)
    }
}

impl<T> Drop for Transaction<T>
where
    T: Connection,
{
    fn drop(&mut self) {
        if self.depth > 0 {
            if let Some(mut inner) = self.inner.take() {
                spawn(async move {
                    let res = inner.execute("ROLLBACK").await;

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
