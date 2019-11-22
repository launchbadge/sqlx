use super::connection::Step;
use super::Postgres;
use super::PostgresQueryParameters;
use super::PostgresRow;
use crate::backend::Backend;
use crate::describe::{Describe, ResultField};
use crate::query::QueryParameters;
use crate::url::Url;
use async_trait::async_trait;
use futures_core::stream::BoxStream;

#[async_trait]
impl Backend for Postgres {
    type QueryParameters = PostgresQueryParameters;

    type Row = PostgresRow;

    type TableIdent = u32;

    async fn open(url: &str) -> crate::Result<Self> {
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

    async fn describe(&mut self, body: &str) -> crate::Result<Describe<Postgres>> {
        self.parse("", body, &PostgresQueryParameters::new());
        self.describe("");
        self.sync().await?;

        let param_desc = loop {
            let step = self
                .step()
                .await?
                .ok_or(invalid_data!("did not receive ParameterDescription"));

            if let Step::ParamDesc(desc) = step? {
                break desc;
            }
        };

        let row_desc = loop {
            let step = self
                .step()
                .await?
                .ok_or(invalid_data!("did not receive RowDescription"));

            if let Step::RowDesc(desc) = step? {
                break desc;
            }
        };

        Ok(Describe {
            param_types: param_desc.ids.into_vec(),
            result_fields: row_desc
                .fields
                .into_vec()
                .into_iter()
                .map(|field| ResultField {
                    name: Some(field.name),
                    table_id: Some(field.table_id),
                    type_id: field.type_id,
                })
                .collect(),
        })
    }
}

impl_from_sql_row_tuples_for_backend!(Postgres);
impl_into_query_parameters_for_backend!(Postgres);
