use futures_core::{future::BoxFuture, stream::BoxStream};
use futures_util::StreamExt;

use crate::{describe::Describe, executor::Executor, pool::Pool, Database};

impl<DB> Executor for Pool<DB>
where
    DB: Database,
{
    type Database = DB;

    fn send<'e, 'q: 'e>(&'e mut self, commands: &'q str) -> BoxFuture<'e, crate::Result<()>> {
        Box::pin(async move { <&Pool<DB> as Executor>::send(&mut &*self, commands).await })
    }

    fn execute<'e, 'q: 'e>(
        &'e mut self,
        query: &'q str,
        args: DB::Arguments,
    ) -> BoxFuture<'e, crate::Result<u64>> {
        Box::pin(async move { <&Pool<DB> as Executor>::execute(&mut &*self, query, args).await })
    }

    fn fetch<'e, 'q: 'e>(
        &'e mut self,
        query: &'q str,
        args: DB::Arguments,
    ) -> BoxStream<'e, crate::Result<DB::Row>> {
        Box::pin(async_stream::try_stream! {
            let mut self_ = &*self;
            let mut s = <&Pool<DB> as Executor>::fetch(&mut self_, query, args);

            while let Some(row) = s.next().await.transpose()? {
                yield row;
            }
        })
    }

    fn fetch_optional<'e, 'q: 'e>(
        &'e mut self,
        query: &'q str,
        args: DB::Arguments,
    ) -> BoxFuture<'e, crate::Result<Option<DB::Row>>> {
        Box::pin(
            async move { <&Pool<DB> as Executor>::fetch_optional(&mut &*self, query, args).await },
        )
    }

    fn describe<'e, 'q: 'e>(
        &'e mut self,
        query: &'q str,
    ) -> BoxFuture<'e, crate::Result<Describe<Self::Database>>> {
        Box::pin(async move { <&Pool<DB> as Executor>::describe(&mut &*self, query).await })
    }
}

impl<DB> Executor for &'_ Pool<DB>
where
    DB: Database,
{
    type Database = DB;

    fn send<'e, 'q: 'e>(&'e mut self, commands: &'q str) -> BoxFuture<'e, crate::Result<()>> {
        Box::pin(async move { self.acquire().await?.send(commands).await })
    }

    fn execute<'e, 'q: 'e>(
        &'e mut self,
        query: &'q str,
        args: DB::Arguments,
    ) -> BoxFuture<'e, crate::Result<u64>> {
        Box::pin(async move { self.acquire().await?.execute(query, args).await })
    }

    fn fetch<'e, 'q: 'e>(
        &'e mut self,
        query: &'q str,
        args: DB::Arguments,
    ) -> BoxStream<'e, crate::Result<DB::Row>> {
        Box::pin(async_stream::try_stream! {
            let mut live = self.acquire().await?;
            let mut s = live.fetch(query, args);

            while let Some(row) = s.next().await.transpose()? {
                yield row;
            }
        })
    }

    fn fetch_optional<'e, 'q: 'e>(
        &'e mut self,
        query: &'q str,
        args: DB::Arguments,
    ) -> BoxFuture<'e, crate::Result<Option<DB::Row>>> {
        Box::pin(async move { self.acquire().await?.fetch_optional(query, args).await })
    }

    fn describe<'e, 'q: 'e>(
        &'e mut self,
        query: &'q str,
    ) -> BoxFuture<'e, crate::Result<Describe<Self::Database>>> {
        Box::pin(async move { self.acquire().await?.describe(query).await })
    }
}
