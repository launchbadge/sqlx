use super::{connection::Step, Postgres};
use crate::{
    backend::Backend,
    describe::{Describe, ResultField},
    executor::Executor,
    params::{IntoQueryParameters, QueryParameters},
    row::FromRow,
    url::Url,
};
use futures_core::{future::BoxFuture, stream::BoxStream};
use crate::postgres::query::PostgresQueryParameters;

impl Executor for Postgres {
    type Backend = Self;

    fn execute<'e, 'q: 'e>(
        &'e mut self,
        query: &'q str,
        params: PostgresQueryParameters,
    ) -> BoxFuture<'e, crate::Result<u64>> {
        Box::pin(async move {
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
        })
    }

    fn fetch<'e, 'q: 'e, T: 'e>(
        &'e mut self,
        query: &'q str,
        params: PostgresQueryParameters,
    ) -> BoxStream<'e, crate::Result<T>>
    where
        T: FromRow<Self::Backend> + Send + Unpin,
    {
        self.parse("", query, &params);
        self.bind("", "", &params);
        self.execute("", 0);

        Box::pin(async_stream::try_stream! {
            self.sync().await?;

            while let Some(step) = self.step().await? {
                if let Step::Row(row) = step {
                    yield FromRow::from_row(row);
                }
            }
        })
    }

    fn fetch_optional<'e, 'q: 'e, T: 'e>(
        &'e mut self,
        query: &'q str,
        params: PostgresQueryParameters,
    ) -> BoxFuture<'e, crate::Result<Option<T>>>
    where
        T: FromRow<Self::Backend> + Send,
    {
        Box::pin(async move {
            self.parse("", query, &params);
            self.bind("", "", &params);
            self.execute("", 2);
            self.sync().await?;

            let mut row: Option<_> = None;

            while let Some(step) = self.step().await? {
                if let Step::Row(r) = step {
                    if row.is_some() {
                        return Err(crate::Error::FoundMoreThanOne);
                    }

                    row = Some(FromRow::from_row(r));
                }
            }

            Ok(row)
        })
    }

    fn describe<'e, 'q: 'e>(
        &'e mut self,
        query: &'q str,
    ) -> BoxFuture<'e, crate::Result<Describe<Self::Backend>>> {
        Box::pin(async move {
            self.parse("", query, &Default::default());
            self.describe("");
            self.sync().await?;

            let param_desc = loop {
                let step = self
                    .step()
                    .await?
                    .ok_or(protocol_err!("did not receive ParameterDescription"));

                if let Step::ParamDesc(desc) = step? {
                    break desc;
                }
            };

            let row_desc = loop {
                let step = self
                    .step()
                    .await?
                    .ok_or(protocol_err!("did not receive RowDescription"));

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
                        name: if field.name == "?column?" { None } else { Some(field.name) },
                        table_id: if field.table_id > 0 { Some(field.table_id) } else { None },
                        type_id: field.type_id,
                        _backcompat: (),
                    })
                    .collect(),
                _backcompat: (),
            })
        })
    }
}
