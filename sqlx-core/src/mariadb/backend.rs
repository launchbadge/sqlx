use super::{MariaDb, MariaDbQueryParameters, MariaDbRow};
use crate::backend::Backend;
use crate::describe::{Describe, ResultField};
use crate::mariadb::protocol::ColumnDefinitionPacket;
use async_trait::async_trait;
use futures_core::stream::BoxStream;

#[async_trait]
impl Backend for MariaDb {
    type QueryParameters = MariaDbQueryParameters;
    type Row = MariaDbRow;
    type TableIdent = String;

    async fn open(url: &str) -> crate::Result<Self>
    where
        Self: Sized,
    {
        MariaDb::open(url).await
    }

    async fn close(mut self) -> crate::Result<()> {
        self.close().await
    }

    async fn ping(&mut self) -> crate::Result<()> {
        self.ping().await
    }

    async fn execute(&mut self, query: &str, params: MariaDbQueryParameters) -> crate::Result<u64> {
        // Write prepare statement to buffer
        self.start_sequence();
        let prepare_ok = self.send_prepare(query).await?;

        let affected = self.execute(prepare_ok.statement_id, params).await?;

        Ok(affected)
    }

    fn fetch(
        &mut self,
        _query: &str,
        _params: MariaDbQueryParameters,
    ) -> BoxStream<'_, crate::Result<MariaDbRow>> {
        unimplemented!();
    }

    async fn fetch_optional(
        &mut self,
        _query: &str,
        _params: MariaDbQueryParameters,
    ) -> crate::Result<Option<Self::Row>> {
        unimplemented!();
    }

    async fn describe(&mut self, query: &str) -> crate::Result<Describe<MariaDb>> {
        let prepare_ok = self.send_prepare(query).await?;

        let mut param_types = Vec::with_capacity(prepare_ok.params as usize);

        for _ in 0..prepare_ok.params {
            let param = ColumnDefinitionPacket::decode(self.receive().await?)?;
            param_types.push(param.field_type.0);
        }

        self.check_eof().await?;

        let mut columns = Vec::with_capacity(prepare_ok.columns as usize);

        for _ in 0..prepare_ok.columns {
            let column = ColumnDefinitionPacket::decode(self.receive().await?)?;
            columns.push(ResultField {
                name: column.column_alias.or(column.column),
                table_id: column.table_alias.or(column.table),
                type_id: column.field_type.0,
            })
        }

        self.check_eof().await?;

        Ok(Describe {
            param_types,
            result_fields: columns,
        })
    }
}

impl_from_sql_row_tuples_for_backend!(MariaDb);
impl_into_query_parameters_for_backend!(MariaDb);
