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

impl<'p, C> Executor<'p> for &'_ Pool<C>
where
    C: 'static + Connect,
    for<'c> &'c mut C: Executor<'c, Database = C::Database>,
{
    type Database = C::Database;

    fn fetch_many<'e, 'q: 'e, E: 'q>(
        self,
        query: E,
    ) -> BoxStream<'e, Result<Either<u64, <Self::Database as Database>::Row>, Error>>
    where
        E: Execute<'q, Self::Database>,
    {
        let pool = self.clone();

        Box::pin(try_stream! {
            let mut conn = pool.acquire().await?;
            let mut s = conn.fetch_many(query);

            for v in s.try_next().await? {
                yield v;
            }
        })
    }

    fn fetch_optional<'e, 'q: 'e, E: 'q>(
        self,
        query: E,
    ) -> BoxFuture<'e, Result<Option<<Self::Database as Database>::Row>, Error>>
    where
        E: Execute<'q, Self::Database>,
    {
        let pool = self.clone();

        Box::pin(async move { pool.acquire().await?.fetch_optional(query).await })
    }

    #[doc(hidden)]
    fn describe<'e, 'q: 'e, E: 'q>(
        self,
        query: E,
    ) -> BoxFuture<'e, Result<Describe<Self::Database>, Error>>
    where
        E: Execute<'q, Self::Database>,
    {
        let pool = self.clone();

        Box::pin(async move { pool.acquire().await?.describe(query).await })
    }
}

// NOTE: required due to lack of lazy normalization
#[allow(unused_macros)]
macro_rules! impl_executor_for_pool_connection {
    ($DB:ident, $C:ident, $R:ident) => {
        impl<'c> crate::executor::Executor<'c> for &'c mut crate::pool::PoolConnection<$C> {
            type Database = $DB;

            #[inline]
            fn fetch_many<'e, 'q: 'e, E: 'q>(
                self,
                query: E,
            ) -> futures_core::stream::BoxStream<
                'e,
                Result<either::Either<u64, $R>, crate::error::Error>,
            >
            where
                'c: 'e,
                E: crate::executor::Execute<'q, $DB>,
            {
                (&mut **self).fetch_many(query)
            }

            #[inline]
            fn fetch_optional<'e, 'q: 'e, E: 'q>(
                self,
                query: E,
            ) -> futures_core::future::BoxFuture<'e, Result<Option<$R>, crate::error::Error>>
            where
                'c: 'e,
                E: crate::executor::Execute<'q, $DB>,
            {
                (&mut **self).fetch_optional(query)
            }

            #[doc(hidden)]
            #[inline]
            fn describe<'e, 'q: 'e, E: 'q>(
                self,
                query: E,
            ) -> futures_core::future::BoxFuture<
                'e,
                Result<crate::describe::Describe<$DB>, crate::error::Error>,
            >
            where
                'c: 'e,
                E: crate::executor::Execute<'q, $DB>,
            {
                (&mut **self).describe(query)
            }
        }
    };
}
