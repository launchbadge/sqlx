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

impl Postgres {
    async fn prepare_cached(&mut self, query: &str, params: &PostgresQueryParameters) -> crate::Result<String> {
        fn get_stmt_name(id: u64) -> String {
            format!("sqlx_postgres_stmt_{}", id)
        }

        let conn = &mut self.conn;
        let next_id = &mut self.next_id;

        self.statements.map_or_compute(
            query,
            |&id| get_stmt_name(id),
            || async {
                let stmt_id = *next_id;
                let stmt_name = get_stmt_name(stmt_id);
                conn.try_parse(&stmt_name, query, params).await?;
                *next_id += 1;
                Ok((stmt_id, stmt_name))
            }).await
    }
}

impl Executor for Postgres {
    type Backend = Self;

    fn execute<'e, 'q: 'e>(
        &'e mut self,
        query: &'q str,
        params: PostgresQueryParameters,
    ) -> BoxFuture<'e, crate::Result<u64>> {
        Box::pin(async move {
            let stmt = self.prepare_cached(query, &params).await?;

            self.conn.bind("", &stmt, &params);
            self.conn.execute("", 1);
            self.conn.sync().await?;

            let mut affected = 0;

            while let Some(step) = self.conn.step().await? {
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
        Box::pin(async_stream::try_stream! {
            let stmt = self.prepare_cached(query, &params).await?;
            self.conn.bind("", &stmt, &params);
            self.conn.execute("", 0);
            self.conn.sync().await?;

            while let Some(step) = self.conn.step().await? {
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
            let stmt = self.prepare_cached(query, &params).await?;
            self.conn.bind("", &stmt, &params);
            self.conn.execute("", 2);
            self.conn.sync().await?;

            let mut row: Option<_> = None;

            while let Some(step) = self.conn.step().await? {
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
            let stmt = self.prepare_cached(query, &PostgresQueryParameters::default()).await?;
            self.conn.describe(&stmt);
            self.conn.sync().await?;

            let param_desc = loop {
                let step = self
                    .conn.step()
                    .await?
                    .ok_or(protocol_err!("did not receive ParameterDescription"));

                if let Step::ParamDesc(desc) = step? {
                    break desc;
                }
            };

            let row_desc = loop {
                let step = self
                    .conn.step()
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
