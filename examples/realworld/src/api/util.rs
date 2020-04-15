use log::*;
use tide::{Request, Response};

use crate::db::model::ProvideError;

/// The signing key used to mint auth tokens
pub const SECRET_KEY: &str = "this-is-the-most-secret-key-ever-secreted";

#[derive(serde::Serialize, serde::Deserialize)]
pub struct TokenClaims {
    pub sub: i32,
    pub exp: i64,
}

/// Retrieve the authorization header from a Request
fn get_auth_header<T>(req: &Request<T>) -> Option<&str> {
    // TODO: It is possible the user will provide multiple auth headers, we should try all of them
    req.header("Authorization")
}

/// Extract the JWT token from a header string
fn parse_token(header: &str) -> String {
    header.splitn(2, ' ').nth(1).unwrap_or_default().to_owned()
}

/// Authorize a JWT returning the user_id
fn authorize_token(token: &str) -> jsonwebtoken::errors::Result<i32> {
    let data = jsonwebtoken::decode::<TokenClaims>(
        token,
        SECRET_KEY.as_ref(),
        &jsonwebtoken::Validation::default(),
    )?;

    Ok(data.claims.sub)
}

/// Validate an auth token if one is present in the request
///
/// This is useful for routes where auth is optional (e.g. /api/get/articles
///
/// 1. No authorization header present -> None
/// 2. Invalid authorization header -> Some(Error)
/// 3. Valid authorization header -> Some(Ok)
pub fn optionally_auth<T>(req: &Request<T>) -> Option<tide::Result<(i32, String)>> {
    if req.headers().contains_key("Authorization") {
        Some(extract_and_validate_token(req))
    } else {
        None
    }
}

/// Validates an auth token from a Request, returning the user ID and token if successful
pub fn extract_and_validate_token<T>(req: &Request<T>) -> tide::Result<(i32, String)> {
    debug!("Checking for auth header");
    let auth_header = get_auth_header(&req)
        .ok_or_else(|| Response::new(400).body_string("Missing Authorization header".to_owned()))?;

    debug!("Extracting token from auth header");
    let token = parse_token(auth_header);

    debug!("Authorizing token");
    let user_id =
        authorize_token(&token).map_err(|e| Response::new(403).body_string(format!("{}", e)))?;

    debug!("Token is valid and belongs to user {}", user_id);

    Ok((user_id, token))
}

/// Converts a serializable payload into a JSON response
///
/// If the body cannot be serialized an Err(Response) will be returned with the serialization error
pub fn to_json_response<B: serde::Serialize>(body: &B) -> Result<Response, Response> {
    Response::new(200).body_json(body).map_err(|e| {
        let error_msg = format!("Failed to serialize response -- {}", e);
        warn!("{}", error_msg);
        Response::new(500).body_string(error_msg)
    })
}

impl From<ProvideError> for Response {
    /// Convert a ProvideError into a [tide::Response]
    ///
    /// This allows the usage of
    fn from(e: ProvideError) -> Response {
        match e {
            ProvideError::NotFound => Response::new(404),
            ProvideError::Provider(e) => Response::new(500).body_string(e.to_string()),
            ProvideError::UniqueViolation(details) => Response::new(409).body_string(details),
            ProvideError::ModelViolation(details) => Response::new(400).body_string(details),
        }
    }
}

impl From<ProvideError> for tide::Error {
    /// Convert a ProvideError into a [tide::Error] via [Response::from]
    ///
    /// This allows the use of the `?` operator in handler functions
    fn from(e: ProvideError) -> Self {
        Response::from(e).into()
    }
}
