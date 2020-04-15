use std::default::Default;

use chrono::{Duration, Utc};
use futures::TryFutureExt;
use log::*;
use rand::{thread_rng, RngCore};
use serde::{Deserialize, Serialize, Deserializer};
use sqlx::pool::PoolConnection;
use sqlx::{Connect, Connection};
use tide::{Error, IntoResponse, Request, Response, ResultExt};

use crate::api::util::{extract_and_validate_token, to_json_response, TokenClaims, SECRET_KEY};
use crate::db::model::{ProvideAuthn, UserEntity};
use crate::db::Db;

/// A User
///
/// [User](https://github.com/gothinkster/realworld/tree/master/api#users-for-authentication)
#[derive(Default, Serialize)]
pub struct User {
    pub email: String,
    pub token: Option<String>,
    pub username: String,
    pub bio: Option<String>,
    pub image: Option<String>,
}

impl User {
    fn token(mut self, token: Option<String>) -> Self {
        self.token = token;
        self
    }
}

/// A field wherein null is significant
///
/// The `realworld` API Spec allows for certain fields to be explicitly set to null
/// (e.g. `image` on [User] objects).
///
/// Serde treats missing values and null values as the same so this type is used to capture
/// that null has meaning. Note that Option<Option<T>> can also be used, however this is slightly
/// more expressive
enum Nullable<T> {
    Data(T),
    Null,
    Missing,
}

impl<T> Nullable<T> {
    /// Converts the field to option if populated or returns `optb`
    ///
    /// Based on [Option::or].
    fn or(self, optb: Option<T>) -> Option<T> {
        match self {
            Nullable::Data(d) => Some(d),
            Nullable::Null => None,
            Nullable::Missing => optb,
        }
    }
}

impl<T> From<Option<T>> for Nullable<T> {
    fn from(opt: Option<T>) -> Self {
        if let Some(data) = opt {
            Nullable::Data(data)
        } else {
            Nullable::Null
        }
    }
}

impl<'de, T> Deserialize<'de> for Nullable<T>
    where T: Deserialize<'de>
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error> where
        D: Deserializer<'de>
    {
        Option::deserialize(deserializer).map(Nullable::from)
    }
}

impl<T> Default for Nullable<T> {
    fn default() -> Self {
        Nullable::Missing
    }
}


/// The response body for User API requests
#[derive(Serialize)]
struct UserResponseBody {
    user: User,
}

impl From<User> for UserResponseBody {
    fn from(user: User) -> Self {
        UserResponseBody { user }
    }
}

impl From<UserEntity> for User {
    fn from(entity: UserEntity) -> Self {
        let UserEntity {
            email,
            username,
            bio,
            image,
            ..
        } = entity;

        User {
            email,
            token: None,
            username,
            bio,
            image,
        }
    }
}

/// Register a new user
///
/// [Registration](https://github.com/gothinkster/realworld/tree/master/api#registration)
pub async fn register(
    mut req: Request<impl Db<Conn = PoolConnection<impl Connect + ProvideAuthn>>>,
) -> Response {
    async {
        #[derive(Deserialize)]
        struct RegisterRequestBody {
            user: NewUser,
        }
        #[derive(Deserialize)]
        struct NewUser {
            username: String,
            email: String,
            password: String,
        }

        // n.b. we don't use req.body_json() because it swallows serde's useful error messages
        let body = req.body_bytes().await.server_err()?;

        let RegisterRequestBody {
            user:
                NewUser {
                    username,
                    email,
                    password,
                },
        } = serde_json::from_slice(&body)
            .map_err(|e| Response::new(400).body_string(e.to_string()))?;

        let hashed_password = hash_password(&password).server_err()?;

        let state = req.state();
        let mut db = state.conn().await.server_err()?;

        let id = db.create_user(&username, &email, &hashed_password).await?;

        // n.b. token creation is a soft-failure as the user can try logging in separately
        let token = generate_token(id)
            .map_err(|e| {
                warn!("Failed to create auth token -- {}", e);
                e
            })
            .ok();

        let user = User {
            email,
            token,
            username,
            bio: None,
            image: None,
        };
        let resp = to_json_response(&UserResponseBody::from(user))?;

        Ok::<_, Error>(resp)
    }
    .await
    .unwrap_or_else(IntoResponse::into_response)
}

/// Get the current user based on their authorization
///
/// [Get Current User](https://github.com/gothinkster/realworld/tree/master/api#get-current-user)
pub async fn get_current_user(
    req: Request<impl Db<Conn = PoolConnection<impl Connect + ProvideAuthn>>>,
) -> Response {
    async move {
        let (user_id, token) = extract_and_validate_token(&req)?;

        let state = req.state();
        let mut db = state.conn().await.server_err()?;

        // n.b - the app doesn't support deleting users
        let user_ent= db.get_user_by_id(user_id).await?;

        let resp = to_json_response(&UserResponseBody::from(User::from(user_ent).token(Some(token))))?;

        Ok::<_, Error>(resp)
    }
    .await
    .unwrap_or_else(IntoResponse::into_response)
}

/// Login to Conduit
///
/// [Login](https://github.com/gothinkster/realworld/tree/master/api#authentication)
pub async fn login(
    mut req: Request<impl Db<Conn = PoolConnection<impl Connect + ProvideAuthn>>>,
) -> Response {
    async move {
        #[derive(Deserialize)]
        struct LoginRequestBody {
            user: Creds,
        }
        #[derive(Deserialize)]
        struct Creds {
            email: String,
            password: String,
        }

        let LoginRequestBody {
            user: Creds { email, password },
        } = req.body_json().await.client_err()?;

        debug!("Parsed login request for {}", &email);

        debug!("Querying DB for user with email {}", &email);
        let state = req.state();
        let mut db = state.conn().await.server_err()?;

        let user_ent = db.get_user_by_email(&email).await.map_err(|e| {
            error!("Failed to get user -- {}", e);
            Response::from(e).set_status(http::StatusCode::FORBIDDEN)
        })?;

        debug!("User {} matches email {}", user_ent.user_id, &email);

        let hashed_password = user_ent.password.as_str();

        debug!("Authenticating user {}", user_ent.user_id);
        let valid =
            argon2::verify_encoded(hashed_password, &password.as_bytes()).with_err_status(403)?;

        if !valid {
            debug!("User {} failed authentication", user_ent.user_id);
            Err(Response::new(403))?
        }

        debug!(
            "Successfully authenticated {}, generating auth token",
            user_ent.user_id
        );
        let token = generate_token(user_ent.user_id).server_err()?;

        let user = User {
            token: Some(token),
            ..user_ent.into()
        };

        let resp = to_json_response(&UserResponseBody::from(user))?;

        Ok::<_, tide::Error>(resp)
    }
    .await
    .unwrap_or_else(IntoResponse::into_response)
}

/// Update a user's email, bio, or image
///
/// [Update User](https://github.com/gothinkster/realworld/tree/master/api#update-user)
pub async fn update_user(
    mut req: Request<impl Db<Conn = PoolConnection<impl Connect + ProvideAuthn>>>,
) -> Response {
    async move {
        #[derive(Deserialize)]
        struct UpdateRequestBody {
            user: UserUpdate,
        }

        #[derive(Deserialize)]
        struct UserUpdate {
            email: Option<String>,
            #[serde(default)]
            bio: Nullable<String>,
            #[serde(default)]
            image: Nullable<String>,
        }
        let (user_id, _) = extract_and_validate_token(&req)?;

        let body = req.body_json().await.server_err()?;

        let state = req.state();
        let mut tx = state
            .conn()
            .and_then(Connection::begin)
            .await
            .server_err()?;

        let updated = {
            let UpdateRequestBody {
                user: UserUpdate { email, bio, image },
            } = body;

            let existing = tx.get_user_by_id(user_id).await?;
            UserEntity {
                email: email.unwrap_or(existing.email),
                bio: bio.or(existing.bio),
                image: image.or(existing.image),
                ..existing
            }
        };

        debug!("Updating user {}", user_id);
        tx.update_user(&updated).await?;
        debug!(
            "Successfully updated user {}. Committing Transaction.",
            user_id
        );
        tx.commit().await.server_err()?;

        let resp = to_json_response(&UserResponseBody::from(User::from(updated)))?;

        Ok::<_, Error>(resp)
    }
    .await
    .unwrap_or_else(IntoResponse::into_response)
}

/// Hashes and salts a password for storage in a DB
fn hash_password(password: &str) -> argon2::Result<String> {
    let salt = generate_random_salt();
    let hash = argon2::hash_encoded(password.as_bytes(), &salt, &argon2::Config::default())?;

    Ok(hash)
}

/// Generate a salt that will be used on passwords
fn generate_random_salt() -> [u8; 16] {
    let mut salt = [0; 16];
    thread_rng().fill_bytes(&mut salt);
    salt
}

/// Generate a JWT for the user_id
fn generate_token(user_id: i32) -> jsonwebtoken::errors::Result<String> {
    use jsonwebtoken::Header;

    let exp = Utc::now() + Duration::hours(24);  // n.b. (bad for sec, good for testing)
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
