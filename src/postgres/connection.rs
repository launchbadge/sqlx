use super::{Postgres, PostgresQueryParameters, PostgresRawConnection, PostgresRow};
use crate::{connection::RawConnection, postgres::raw::Step, url::Url};
use async_trait::async_trait;
use futures_core::stream::BoxStream;

#[async_trait]
impl RawConnection for PostgresRawConnection {
    type Backend = Postgres;

    async fn establish(url: &str) -> crate::Result<Self> {
        let url = Url::parse(url)?;
        let address = url.resolve(5432);
        let mut conn = Self::new(address).await?;

        conn.startup(
            url.username(),
            url.password().unwrap_or_default(),
            url.database(),
        )
        .await?;

        Ok(conn)
    }

    #[inline]
    async fn close(mut self) -> crate::Result<()> {
        self.terminate().await
    }

    async fn execute(
        &mut self,
        query: &str,
        params: PostgresQueryParameters,
    ) -> crate::Result<u64> {
        self.parse("", query, &params);
        self.bind("", "", &params);
        self.execute("", 1);
        self.sync().await?;

        let mut affected = 0;

        while let Some(step) = self.step().await? {
            if let Step::Command(cnt) = step {
                affected = cnt;
            }
        }

        Ok(affected)
    }

    fn fetch(
        &mut self,
        query: &str,
        params: PostgresQueryParameters,
    ) -> BoxStream<'_, crate::Result<PostgresRow>> {
        self.parse("", query, &params);
        self.bind("", "", &params);
        self.execute("", 0);

        Box::pin(async_stream::try_stream! {
            self.sync().await?;

            while let Some(step) = self.step().await? {
                if let Step::Row(row) = step {
                    yield row;
                }
            }
        })
    }

    async fn fetch_optional(
        &mut self,
        query: &str,
        params: PostgresQueryParameters,
    ) -> crate::Result<Option<PostgresRow>> {
        self.parse("", query, &params);
        self.bind("", "", &params);
        self.execute("", 2);
        self.sync().await?;

        let mut row: Option<PostgresRow> = None;

        while let Some(step) = self.step().await? {
            if let Step::Row(r) = step {
                if row.is_some() {
                    return Err(crate::Error::FoundMoreThanOne);
                }

                row = Some(r);
            }
        }

        Ok(row)
    }
}
