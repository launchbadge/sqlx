use std::ops::{Deref, DerefMut};

use futures_core::future::BoxFuture;

use crate::connection::Connection;
use crate::database::Database;
use crate::database::HasCursor;
use crate::describe::Describe;
use crate::executor::{Execute, Executor, RefExecutor};
use crate::runtime::spawn;

/// Represents a database transaction.
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
    pub(crate) async fn new(depth: u32, mut inner: T) -> crate::Result<T::Database, Self> {
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

    /// Creates a new save point in the current transaction and returns
    /// a new `Transaction` object to manage its scope.
    pub async fn begin(self) -> crate::Result<T::Database, Transaction<Transaction<T>>> {
        Transaction::new(self.depth, self).await
    }

    /// Commits the current transaction or save point.
    /// Returns the inner connection or transaction.
    pub async fn commit(mut self) -> crate::Result<T::Database, T> {
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

    /// Rollback the current transaction or save point.
    /// Returns the inner connection or transaction.
    pub async fn rollback(mut self) -> crate::Result<T::Database, T> {
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

impl<T> Connection for Transaction<T>
where
    T: Connection,
{
    // Close is equivalent to
    fn close(mut self) -> BoxFuture<'static, crate::Result<T::Database, ()>> {
        Box::pin(async move {
            let mut inner = self.inner.take().expect(ERR_FINALIZED);

            if self.depth == 1 {
                // This is the root transaction, call rollback
                let res = inner.execute("ROLLBACK").await;

                // No matter the result of the above, call close
                let _ = inner.close().await;

                // Now raise the error if there was one
                res?;
            } else {
                // This is not the root transaction, forward to a nested
                // transaction (to eventually call rollback)
                inner.close().await?
            }

            Ok(())
        })
    }

    #[inline]
    fn ping(&mut self) -> BoxFuture<'_, crate::Result<T::Database, ()>> {
        self.deref_mut().ping()
    }
}

impl<DB, T> Executor for Transaction<T>
where
    DB: Database,
    T: Connection<Database = DB>,
{
    type Database = T::Database;

    fn execute<'e, 'q: 'e, 'c: 'e, E: 'e>(
        &'c mut self,
        query: E,
    ) -> BoxFuture<'e, crate::Result<T::Database, u64>>
    where
        E: Execute<'q, Self::Database>,
    {
        (**self).execute(query)
    }

    fn fetch<'e, 'q, E>(&'e mut self, query: E) -> <Self::Database as HasCursor<'e, 'q>>::Cursor
    where
        E: Execute<'q, Self::Database>,
    {
        (**self).fetch(query)
    }

    fn describe<'e, 'q, E: 'e>(
        &'e mut self,
        query: E,
    ) -> BoxFuture<'e, crate::Result<T::Database, Describe<Self::Database>>>
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
            if let Some(inner) = self.inner.take() {
                spawn(async move {
                    let _ = inner.close().await;
                });
            }
        }
    }
}
