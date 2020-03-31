use chrono::{Duration, Utc};
use log::*;
use rand::{thread_rng, RngCore};
use tide::{Request, Response, IntoResponse};

use super::{ApiResult, ApiError};
use crate::db::model::{ProvideUser, UserEntity};
use std::default::Default;

const SECRET_KEY: &str = "this-is-the-most-secret-key-ever-secreted";

// User
// https://github.com/gothinkster/realworld/tree/master/api#users-for-authentication
#[derive(Default, serde::Serialize)]
pub struct User {
    pub email: String,
    pub token: Option<String>,
    pub username: String,
    pub bio: Option<String>,
    pub image: Option<String>,
}

// Registration
// https://github.com/gothinkster/realworld/tree/master/api#registration

// #[post("/api/users")]
pub async fn register(req: Request<impl ProvideUser>) -> Response {
    async fn inner(mut req: Request<impl ProvideUser>) -> ApiResult<Response> {
        #[derive(serde::Deserialize)]
        struct RegisterRequestBody {
            user: NewUser
        }
        #[derive(serde::Deserialize)]
        struct NewUser {
            username: String,
            email: String,
            password: String,
        }

        let RegisterRequestBody {user: NewUser {
            username, email, password
        }} = req.body_json().await
            .map_err(|e| ApiError::Api(Response::new(400).body_string(e.to_string())))?;

        let hashed_password = hash_password(&password)?;

        let db = req.state();
        let id = db.create_user(&username, &email, &hashed_password).await?;

        // This is not a hard failure, the user should simply try to login
        let token = generate_token(id)
            .map_err(|e| {
                warn!("Failed to create auth token -- {}", e);
                e
            })
            .ok();

        #[derive(serde::Serialize)]
        struct RegisterResponseBody {
            user: User,
        }

        let resp = Response::new(200)
            .body_json(&RegisterResponseBody {
                user: User {
                    email,
                    token,
                    username,
                    ..Default::default()
                }
            })
            .map_err(anyhow::Error::from)?;

        Ok(resp)
    }
    inner(req).await.unwrap_or_else(IntoResponse::into_response)
}

// Get Current User
// https://github.com/gothinkster/realworld/tree/master/api#get-current-user

// #[get("/api/user")]
pub async fn get_current_user(req: Request<impl ProvideUser>) -> Response {
    async fn inner(req: Request<impl ProvideUser>) -> ApiResult<Response> {

        // FIXME(sgg): Replace this with an auth middleware?
        let auth_header = req.header("authorization")
            .ok_or_else(|| {
                ApiError::Api(Response::new(400).body_string("Missing Authorization header".to_owned()))
            })?;

        let token = get_token_from_request(auth_header);

        let user_id = authorize(&token).await
            .map_err(|e| ApiError::Api(Response::new(403).body_string(format!("{}", e))))?;

        debug!("Token is authorized to user {}", user_id);

        let db = req.state();

        let UserEntity { email, username, .. } = db.get_user_by_id(user_id).await?;

        #[derive(serde::Serialize)]
        struct GetCurrentUserResponseBody {
            user: User,
        }

        let resp = Response::new(200)
            .body_json(&GetCurrentUserResponseBody {
                user: User {
                    email,
                    token: Some(token.to_owned()),
                    username,
                    ..Default::default()
                },
            })
            .map_err(anyhow::Error::from)?;

        Ok(resp)
    }
    inner(req).await.unwrap_or_else(IntoResponse::into_response)
}

// Login
// https://github.com/gothinkster/realworld/tree/master/api#authentication
pub async fn login(req: Request<impl ProvideUser>) -> Response {
    async fn inner(mut req: Request<impl ProvideUser>) -> ApiResult<Response> {
        #[derive(serde::Deserialize)]
        struct LoginRequestBody {
            user: Creds
        }
        #[derive(serde::Deserialize)]
        struct Creds {
            email: String,
            password: String,
        }

        let LoginRequestBody {user: Creds { email, password }} = req.
            body_json()
            .await
            .map_err(|_| Response::new(400))?;
        debug!("Parsed login request for {}", &email);

        debug!("Querying DB for user with email {}", &email);
        let db = req.state();
        let user = db.get_user_by_email(&email)
            .await
            .map_err(|e| {
                error!("Failed to get user -- {}", e);
                e
            })?;

        debug!("User {} matches email {}", user.id, &email);

        let hashed_password = user.password.as_ref()
            .ok_or_else(|| Response::new(403))?;

        debug!("Authenticating user {}", user.id);
        let valid = argon2::verify_encoded(hashed_password, &password.as_bytes())
            .map_err(|_| Response::new(403))?;

        if ! valid {
            debug!("User {} failed authentication", user.id);
            Err(Response::new(403))?
        }

        debug!("Successfully authenticated {}, generating auth token", user.id);
        let token = generate_token(user.id)?;

        #[derive(serde::Serialize)]
        struct LoginResponseBody {
            user: User
        }

        let resp = to_json_response(&LoginResponseBody {
            user: User {
                email,
                token: Some(token),
                username: user.username,
                ..Default::default()
            }
        })?;
        Ok(resp)
    }
    inner(req).await.unwrap_or_else(IntoResponse::into_response)
}


/// Converts a serializable payload into a JSON response
fn to_json_response<B: serde::Serialize>(body: &B) -> Result<Response, Response> {
    Response::new(200)
        .body_json(body)
        .map_err(|e| {
            let error_msg = format!("Failed to serialize response -- {}", e);
            warn!("{}", error_msg);
            Response::new(500).body_string(error_msg)
        })
}

fn get_token_from_request(header: &str) -> String {
    header
        .splitn(2, ' ')
        .nth(1)
        .unwrap_or_default()
        .to_owned()
}

async fn authorize(token: &str) -> anyhow::Result<i32> {
    let data = jsonwebtoken::decode::<TokenClaims>(
        token,
        SECRET_KEY.as_ref(),
        &jsonwebtoken::Validation::default(),
    )?;

    Ok(data.claims.sub)
}

// TODO: Does this need to be spawned in async-std ?
fn hash_password(password: &str) -> anyhow::Result<String> {
    let salt = generate_random_salt();
    let hash = argon2::hash_encoded(password.as_bytes(), &salt, &argon2::Config::default())?;

    Ok(hash)
}

fn generate_random_salt() -> [u8; 16] {
    let mut salt = [0; 16];
    thread_rng().fill_bytes(&mut salt);

    salt
}

#[derive(serde::Serialize, serde::Deserialize)]
struct TokenClaims {
    sub: i32,
    exp: i64,
}

fn generate_token(user_id: i32) -> anyhow::Result<String> {
    use jsonwebtoken::Header;

    let exp = Utc::now() + Duration::hours(1);
    let token = jsonwebtoken::encode(
        &Header::default(),
        &TokenClaims {
            sub: user_id,
            exp: exp.timestamp(),
        },
        SECRET_KEY.as_ref(),
    )?;

    Ok(token)
}
