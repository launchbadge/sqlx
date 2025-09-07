use anyhow::Context;
use axum::{extract::FromRef, Router};
use sqlx::PgPool;
use tokio::net::TcpListener;

mod error;

mod post;
mod user;

pub use self::error::Error;

pub type Result<T, E = Error> = ::std::result::Result<T, E>;

#[derive(Clone, FromRef)]
pub struct AppState {
    db: PgPool,
}

pub fn app(db: PgPool) -> Router {
    Router::new()
        .merge(user::router())
        .merge(post::router())
        .with_state(AppState { db })
}

pub async fn serve(db: PgPool) -> anyhow::Result<()> {
    let listener = TcpListener::bind("0.0.0.0:8080").await?;
    axum::serve(listener, app(db))
        .await
        .context("failed to serve API")
}
