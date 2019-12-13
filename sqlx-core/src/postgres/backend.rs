use super::{connection::{PostgresConn, Step}, Postgres};
use crate::{
    backend::Backend,
    describe::{Describe, ResultField},
    params::QueryParameters,
    postgres::{protocol::DataRow, query::PostgresQueryParameters},
    url::Url,
};
use futures_core::{future::BoxFuture, stream::BoxStream};
use crate::cache::StatementCache;

impl Backend for Postgres {
    type QueryParameters = PostgresQueryParameters;

    type Row = DataRow;

    type TableIdent = u32;

    fn open(url: &str) -> BoxFuture<'static, crate::Result<Self>> {
        let url = Url::parse(url);

        Box::pin(async move {
            let url = url?;
            let address = url.resolve(5432);
            let mut conn = PostgresConn::new(address).await?;

            conn.startup(
                url.username(),
                url.password().unwrap_or_default(),
                url.database(),
            )
            .await?;

            Ok(Postgres {
                conn,
                statements: StatementCache::new(),
                next_id: 0
            })
        })
    }

    fn close(self) -> BoxFuture<'static, crate::Result<()>> {
        Box::pin(self.conn.terminate())
    }
}

impl_from_row_for_backend!(Postgres, DataRow);
impl_into_query_parameters_for_backend!(Postgres);
