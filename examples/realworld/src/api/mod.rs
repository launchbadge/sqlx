use log::*;
use tide::{IntoResponse, Response};

/// Route handlers for the /api/articles APIs
pub mod articles;

/// Route handlers for the /user(s) APIs
pub mod users;

/// A shim error that enables ergonomic error handling w/ Tide
#[derive(Debug, thiserror::Error)]
pub enum ApiError {
    #[error("Status Code {}", .0.status())]
    Api(Response),
    #[error(transparent)]
    Other(#[from] anyhow::Error),
}

type ApiResult<T> = Result<T, ApiError>;

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        match self {
            ApiError::Api(r) => r,
            ApiError::Other(e) => {
                Response::new(500).body_string(format!("Unexpected error -- {}", e))
            }
        }
    }
}

impl From<Response> for ApiError {
    fn from(resp: Response) -> Self {
        ApiError::Api(resp)
    }
}
