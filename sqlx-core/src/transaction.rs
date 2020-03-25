use std::ops::{Deref, DerefMut};

use futures_core::future::BoxFuture;

use crate::connection::Connection;
use crate::cursor::HasCursor;
use crate::database::Database;
use crate::describe::Describe;
use crate::executor::{Execute, Executor, RefExecutor};
use crate::runtime::spawn;

/// Represents an in-progress database transaction.
///
/// A transaction ends with a call to [`commit`] or [`rollback`] in which the wrapped connection (
/// or outer transaction) is returned. If neither are called before the transaction
/// goes out-of-scope, [`rollback`] is called. In other words, [`rollback`] is called on `drop`
/// if the transaction is still in-progress.
///
/// ```rust,ignore
/// // Acquire a new connection and immediately begin a transaction
/// let mut tx = pool.begin().await?;
///
/// sqlx::query("INSERT INTO articles (slug) VALUES ('this-is-a-slug')")
///     .execute(&mut tx)
///     // As we didn't fill in all the required fields in this INSERT,
///     // this statement will fail. Since we used `?`, this function
///     // will immediately return with the error which will cause
///     // this transaction to be rolled back.
///     .await?;
/// ```
///
/// [`commit`]: #method.commit
/// [`rollback`]: #method.rollback
// Transaction<PoolConnection<PgConnection>>
// Transaction<PgConnection>
pub struct Transaction<C>
where
    C: Connection,
{
    inner: Option<C>,
    depth: u32,
}

impl<C> Transaction<C>
where
    C: Connection,
{
    pub(crate) async fn new(depth: u32, mut inner: C) -> crate::Result<C::Database, Self> {
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
    pub async fn begin(self) -> crate::Result<C::Database, Transaction<Transaction<C>>> {
        Transaction::new(self.depth, self).await
    }

    /// Commits the current transaction or save point.
    /// Returns the inner connection or transaction.
    pub async fn commit(mut self) -> crate::Result<C::Database, C> {
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
    pub async fn rollback(mut self) -> crate::Result<C::Database, C> {
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

impl<C> Deref for Transaction<C>
where
    C: Connection,
{
    type Target = C;

    fn deref(&self) -> &Self::Target {
        self.inner.as_ref().expect(ERR_FINALIZED)
    }
}

impl<C> DerefMut for Transaction<C>
where
    C: Connection,
{
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.inner.as_mut().expect(ERR_FINALIZED)
    }
}

impl<C> Connection for Transaction<C>
where
    C: Connection,
{
    // Close is equivalent to
    fn close(mut self) -> BoxFuture<'static, crate::Result<C::Database, ()>> {
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
    fn ping(&mut self) -> BoxFuture<'_, crate::Result<C::Database, ()>> {
        self.deref_mut().ping()
    }
}

impl<DB, C> Executor for Transaction<C>
where
    DB: Database,
    C: Connection<Database = DB>,
{
    type Database = C::Database;

    fn execute<'e, 'q: 'e, 'c: 'e, E: 'e>(
        &'c mut self,
        query: E,
    ) -> BoxFuture<'e, crate::Result<C::Database, u64>>
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

    #[doc(hidden)]
    fn describe<'e, 'q, E: 'e>(
        &'e mut self,
        query: E,
    ) -> BoxFuture<'e, crate::Result<C::Database, Describe<Self::Database>>>
    where
        E: Execute<'q, Self::Database>,
    {
        (**self).describe(query)
    }
}

impl<'e, DB, C> RefExecutor<'e> for &'e mut Transaction<C>
where
    DB: Database,
    C: Connection<Database = DB>,
{
    type Database = DB;

    fn fetch_by_ref<'q, E>(self, query: E) -> <Self::Database as HasCursor<'e, 'q>>::Cursor
    where
        E: Execute<'q, Self::Database>,
    {
        (**self).fetch(query)
    }
}

impl<C> Drop for Transaction<C>
where
    C: Connection,
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
