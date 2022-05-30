use std::sync::Arc;

use futures_core::future::BoxFuture;

use crate::error::Error;

#[derive(Clone)]
pub enum PgPassword {
    Static(String),
    Dynamic(Arc<dyn Fn() -> BoxFuture<'static, Result<String, Error>> + 'static + Send + Sync>),
}

impl PgPassword {
    pub async fn password(&self) -> Result<String, Error> {
        match &self {
            PgPassword::Static(password) => Ok(password.clone()),
            PgPassword::Dynamic(closure) => closure().await,
        }
    }
}

impl std::fmt::Debug for PgPassword {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        todo!()
    }
}
