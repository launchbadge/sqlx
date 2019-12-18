use futures_core::{future::BoxFuture, stream::BoxStream};

use crate::{
    backend::Backend,
    describe::{Describe, ResultField},
    params::QueryParameters,
    postgres::{protocol::DataRow, query::PostgresQueryParameters},
    url::Url,
};
use crate::cache::StatementCache;

use super::{Connection, RawConnection, Postgres};

impl Backend for Postgres {
    type Connection = Connection;

    type QueryParameters = PostgresQueryParameters;

    type Row = DataRow;

    type TableIdent = u32;

    fn connect(url: &str) -> BoxFuture<'static, crate::Result<Connection>> {
        let url = Url::parse(url);

        Box::pin(async move {
            let url = url?;
            let address = url.resolve(5432);
            let mut conn = RawConnection::new(address).await?;

            conn.startup(
                url.username(),
                url.password().unwrap_or_default(),
                url.database(),
            )
            .await?;

            Ok(Connection {
                conn,
                statements: StatementCache::new(),
                next_id: 0
            })
        })
    }
}

impl_from_row_for_backend!(Postgres, DataRow);
impl_into_query_parameters_for_backend!(Postgres);
