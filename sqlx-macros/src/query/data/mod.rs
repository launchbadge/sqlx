use crate::database::DatabaseExt;
use sqlx_core::database::Database;
use sqlx_core::describe::Describe;
use sqlx_core::executor::Executor;

#[cfg(feature = "offline")]
pub mod offline;

#[cfg_attr(feature = "offline", derive(serde::Serialize))]
#[cfg_attr(
    feature = "offline",
    serde(bound(serialize = "Describe<DB>: serde::Serialize",))
)]
#[derive(Debug)]
pub struct QueryData<DB: DatabaseExt> {
    #[allow(dead_code)]
    pub(super) query: String,
    pub(super) describe: Describe<DB>,
    #[cfg(feature = "offline")]
    pub(super) hash: String,
    #[cfg(feature = "offline")]
    db_name: offline::SerializeDbName<DB>,
}

impl<DB: DatabaseExt> QueryData<DB> {
    pub async fn from_db(
        conn: impl Executor<'_, Database = DB>,
        query: &str,
    ) -> crate::Result<Self> {
        Ok(QueryData {
            query: query.into(),
            describe: conn.describe(query).await?,
            #[cfg(feature = "offline")]
            hash: super::hash_string(query),
            #[cfg(feature = "offline")]
            db_name: offline::SerializeDbName::default(),
        })
    }
}
