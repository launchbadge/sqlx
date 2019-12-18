use futures_core::{future::BoxFuture, stream::BoxStream};

use crate::{
    backend::Backend,
    describe::{Describe, ResultField},
    mysql::{protocol::ResultRow, query::MariaDbQueryParameters},
    url::Url,
};

use super::{Connection, RawConnection};
use super::MySql;
use crate::cache::StatementCache;

impl Backend for MySql {
    type Connection = Connection;
    type QueryParameters = MariaDbQueryParameters;
    type Row = ResultRow;
    type TableIdent = String;

    fn connect(url: &str) -> BoxFuture<'static, crate::Result<Connection>> {
        let url = Url::parse(url);

        Box::pin(async move {
            let url = url?;
            Ok(Connection {
                conn: RawConnection::open(url).await?,
                cache: StatementCache::new(),
            })
        })
    }
}

impl_from_row_for_backend!(MySql, ResultRow);
impl_into_query_parameters_for_backend!(MySql);
