use super::{connection::Step, Postgres, PostgresQueryParameters, PostgresRow};
use crate::{
    backend::Backend,
    describe::{Describe, ResultField},
    params::QueryParameters,
    url::Url,
};
use futures_core::{future::BoxFuture, stream::BoxStream};

impl Backend for Postgres {
    type QueryParameters = PostgresQueryParameters;

    type Row = PostgresRow;

    type TableIdent = u32;

    fn open(url: &str) -> BoxFuture<'static, crate::Result<Self>> {
        let url = Url::parse(url);

        Box::pin(async move {
            let url = url?;
            let address = url.resolve(5432);
            let mut conn = Self::new(address).await?;

            conn.startup(
                url.username(),
                url.password().unwrap_or_default(),
                url.database(),
            )
            .await?;

            Ok(conn)
        })
    }

    fn close(mut self) -> BoxFuture<'static, crate::Result<()>> {
        Box::pin(self.terminate())
    }
}

impl_from_row_for_backend!(Postgres);
impl_into_query_parameters_for_backend!(Postgres);
