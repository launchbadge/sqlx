use async_stream::try_stream;
use either::Either;
use futures_core::future::BoxFuture;
use futures_core::stream::BoxStream;
use futures_util::TryStreamExt;

use crate::connection::Connect;
use crate::database::Database;
use crate::describe::Describe;
use crate::error::Error;
use crate::executor::{Execute, Executor};
use crate::pool::Pool;

impl<'p, C> Executor<'p> for &'p Pool<C>
where
    C: Connect,
    for<'c> &'c mut C: Executor<'c, Database = C::Database>,
{
    type Database = C::Database;

    fn fetch_many<'q, E: 'p>(
        self,
        query: E,
    ) -> BoxStream<'p, Result<Either<u64, <Self::Database as Database>::Row>, Error>>
    where
        E: Execute<'q, Self::Database>,
    {
        Box::pin(try_stream! {
            let mut conn = self.acquire().await?;
            let mut s = conn.fetch_many(query);

            for v in s.try_next().await? {
                yield v;
            }
        })
    }

    fn fetch_optional<'q, E: 'p>(
        self,
        query: E,
    ) -> BoxFuture<'p, Result<Option<<Self::Database as Database>::Row>, Error>>
    where
        E: Execute<'q, Self::Database>,
    {
        Box::pin(async move { self.acquire().await?.fetch_optional(query).await })
    }

    #[doc(hidden)]
    fn describe<'q, E: 'p>(self, query: E) -> BoxFuture<'p, Result<Describe<Self::Database>, Error>>
    where
        E: Execute<'q, Self::Database>,
    {
        Box::pin(async move { self.acquire().await?.describe(query).await })
    }
}

// NOTE: required due to lack of lazy normalization
macro_rules! impl_executor_for_pool_connection {
    ($DB:ident, $C:ident, $R:ident) => {
        impl<'c> crate::executor::Executor<'c> for &'c mut crate::pool::PoolConnection<$C> {
            type Database = $DB;

            #[inline]
            fn fetch_many<'q: 'c, E: 'c>(
                self,
                query: E,
            ) -> futures_core::stream::BoxStream<
                'c,
                Result<either::Either<u64, $R>, crate::error::Error>,
            >
            where
                E: crate::executor::Execute<'q, $DB>,
            {
                (&mut **self).fetch_many(query)
            }

            #[inline]
            fn fetch_optional<'q: 'c, E: 'c>(
                self,
                query: E,
            ) -> futures_core::future::BoxFuture<'c, Result<Option<$R>, crate::error::Error>>
            where
                E: crate::executor::Execute<'q, $DB>,
            {
                (&mut **self).fetch_optional(query)
            }

            #[doc(hidden)]
            #[inline]
            fn describe<'q: 'c, E: 'c>(
                self,
                query: E,
            ) -> futures_core::future::BoxFuture<
                'c,
                Result<crate::describe::Describe<$DB>, crate::error::Error>,
            >
            where
                E: crate::executor::Execute<'q, $DB>,
            {
                (&mut **self).describe(query)
            }
        }
    };
}
