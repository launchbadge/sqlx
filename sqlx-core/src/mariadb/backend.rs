use super::MariaDb;
use crate::mariadb::protocol::ResultRow;
use crate::mariadb::query::MariaDbQueryParameters;
use crate::{
    backend::Backend,
    describe::{Describe, ResultField},
};
use futures_core::stream::BoxStream;
use futures_core::future::BoxFuture;
use crate::url::Url;

impl Backend for MariaDb {
    type QueryParameters = MariaDbQueryParameters;
    type Row = ResultRow;
    type TableIdent = String;

    fn open(url: &str) -> BoxFuture<'static, crate::Result<Self>> {
        let url = Url::parse(url);

        Box::pin(async move {
            let url = url?;
            MariaDb::open(url).await
        })
    }

    fn close(self) -> BoxFuture<'static, crate::Result<()>> {
        Box::pin(async move {
            self.close().await
        })
    }
}

impl_from_row_for_backend!(MariaDb);
impl_into_query_parameters_for_backend!(MariaDb);
